---
title: Ansible is a Lisp
tagline: More specifically, Ansible is homoiconic and has syntactic macros
tags:
  - ansible
  - lisp
  - macros
slug:
  ordinal: 0
  name: ansible-is-a-lisp
date:
  created: 2024-05-01 12:27:28-07:00
  published: 2024-05-01 13:14:44-07:00
---

The Lisp family of languages is unique because code and data have the exact same
form, and code is data, and data is code. This property is called
[homoiconicity](https://en.wikipedia.org/wiki/Homoiconicity).

Consider the following Racket program (from
[The Racket Guide](<https://docs.racket-lang.org/guide/intro.html#(part._.Creating_.Executables)>)):

```lisp
(define (extract str)
  (substring str 4 7))

(extract "the cat out of the bag")
```

When evaluated by Racket, it will output "cat". However, you can also think of
it as a pure piece of data -- a nested list of strings and numbers and other
objects. If you aren't familiar with Lisp syntax, here's a JSON equivalent
(though Racket has symbols and JSON doesn't so we'll turn symbols into strings):

<!-- prettier-ignore-start -->
```json
[["define", ["extract", "str"],
  ["substring", "str", 4, 7]],

["extract", "the cat out of the bag"]]
```
<!-- prettier-ignore-end -->

However, [Ansible](https://www.ansible.com/), the IaC automation tool, is also
homoiconic. It executes YAML files, and can treat YAML as data.
[Here's an example playbook as a hello world, pulled from Ansible's own introduction page](https://docs.ansible.com/ansible/latest/getting_started/get_started_playbook.html):

```yml
- name: My first play
  hosts: myhosts
  tasks:
    - name: Ping my hosts
      ansible.builtin.ping:

    - name: Print message
      ansible.builtin.debug:
        msg: Hello world
```

Ansible also has syntactic JSON templating, as seen in
[this StackOverflow answer](https://stackoverflow.com/a/63450413):

```yml
- copy:
    dest: kube-controller-manager-csr.json
    content: "{{ certificate | to_json }}"
  vars:
    certificate:
      CN: system:kube-controller-manager
      key:
        algo: rsa
        size: 2048
      names:
        - C: US
          L: Portland
          O: system:kube-controller-manager
          OU: Kubernetes The Hard Way
          ST: Oregon
```

This writes the following file:

```json
{
  "CN": "system:kube-controller-manager",
  "key": { "algo": "rsa", "size": 2048 },
  "names": [
    {
      "C": "US",
      "L": "Portland",
      "O": "system:kube-controller-manager",
      "OU": "Kubernetes The Hard Way",
      "ST": "Oregon"
    }
  ]
}
```

## Syntactic Macros

One of the killer features of Lisp is that it has
[syntactic macros](<https://en.wikipedia.org/wiki/Macro_(computer_science)#Syntactic_macros>).
However, Ansible also has those. Consider the following playbook:

```yml
- name: ansible has syntactic macros ehe :3
  hosts: ::1
  connection: local
  tasks:
    - name: Run a shell command and register its output to foo_result
      ansible.builtin.shell: echo myfoo first
      register: foo_result

    - name: Syntactically generate some tasks
      copy:
        dest: intermediate.generated.yml
        content: "{{ tasks | to_json }}"
      vars:
        tasks:
          - name: Print something silly
            ansible.builtin.debug:
              msg: "foo_result was templated in as: {{ foo_result.stdout }}"

    - name: Set foo_result to something else to test scoping
      ansible.builtin.shell: echo myfoo second
      register: foo_result

    - name: Execute the generated tasks
      ansible.builtin.include_tasks: intermediate.generated.yml
```

Output:

```
$ ansible-playbook playbook.yml
[WARNING]: No inventory was parsed, only implicit localhost is available
[WARNING]: provided hosts list is empty, only localhost is available. Note that
the implicit localhost does not match 'all'

PLAY [ansible has syntactic macros ehe :3] *************************************

TASK [Gathering Facts] *********************************************************
ok: [::1]

TASK [Run a shell command and register its output to foo_result] ***************
changed: [::1]

TASK [Syntactically generate some tasks] ***************************************
ok: [::1]

TASK [Set foo_result to something else to test scoping] ************************
changed: [::1]

TASK [Execute the generated tasks] *********************************************
included: /home/astrid/Documents/lispible/intermediate.generated.yml for ::1

TASK [Print something silly] ***************************************************
ok: [::1] => {
    "msg": "foo_result was templated in as: myfoo first"
}

PLAY RECAP *********************************************************************
::1                        : ok=6    changed=2    unreachable=0    failed=0    skipped=0    rescued=0    ignored=0
```

What's going on here?

Here's the part where we generate the syntax:

```yml
- name: Syntactically generate some tasks
  copy:
    dest: intermediate.generated.yml
    content: "{{ tasks | to_yaml }}"
  vars:
    tasks:
      - name: Print something silly
        ansible.builtin.debug:
          msg: "foo_result was templated in as: {{ foo_result.stdout }}"
```

Essentially, we are syntactically constructing a task inside the `copy` task,
and writing it to a file called `intermediate.generated.yml`. Here's what it
looks like:

```yml
- ansible.builtin.debug: { msg: "foo_result was templated in as: myfoo first" }
  name: Print something silly
```

You'll notice that we string-templated in the first evaluation of `foo_result`,
so that's the one captured here

Here's where we execute the generated syntax:

```yml
- name: Execute the generated tasks
  ansible.builtin.include_tasks: intermediate.generated.yml
```

This is actually done when the task is evaluated, rather than when the whole
program is loaded, so this will be able to find the generated YAML file.

Unfortunately, these macros are non-hygenic, so if we did something like this
instead:

```yml
- name: ansible has syntactic macros ehe :3
  hosts: ::1
  connection: local
  tasks:
    - name: Run a shell command and register its output to foo_result
      ansible.builtin.shell: echo myfoo first
      register: foo_result

    - name: Syntactically generate some tasks
      copy:
        dest: intermediate.generated.yml
        content: "{{ tasks | to_yaml }}"
      vars:
        tasks:
          - name: Print something silly
            ansible.builtin.debug:
              msg:
                # Note this line here!
                "foo_result was templated in as: {{ '{{ foo_result.stdout }}' }}"

    - name: Set foo_result to something else to test scoping
      ansible.builtin.shell: echo myfoo second
      register: foo_result

    - name: Execute the generated tasks
      ansible.builtin.include_tasks: intermediate.generated.yml
```

we would get the following intermediate file:

```yml
- ansible.builtin.debug:
    { msg: "foo_result was templated in as: {{ foo_result.stdout }}" }
  name: Print something silly
```

which will capture the second evaluation of foo_result, as seen in this output:

```
> ansible-playbook playbook2.yml
[WARNING]: No inventory was parsed, only implicit localhost is available
[WARNING]: provided hosts list is empty, only localhost is available. Note that the implicit localhost does
not match 'all'

PLAY [ansible has syntactic macros ehe :3] *****************************************************************

TASK [Gathering Facts] *************************************************************************************
ok: [::1]

TASK [Run a shell command and register its output to foo_result] *******************************************
changed: [::1]

TASK [Syntactically generate some tasks] *******************************************************************
ok: [::1]

TASK [Set foo_result to something else to test scoping] ****************************************************
changed: [::1]

TASK [Execute the generated tasks] *************************************************************************
included: /home/astrid/Documents/lispible/intermediate.generated.yml for ::1

TASK [Print something silly] *******************************************************************************
ok: [::1] => {
    "msg": "foo_result was templated in as: myfoo second"
}

PLAY RECAP *************************************************************************************************
::1                        : ok=6    changed=2    unreachable=0    failed=0    skipped=0    rescued=0    ignored=0
```

## Where are the parens, though?

"But wait," you, the rhetorical Lisp purist, might ask. "Ansible doesn't have
all those funny parens that Lisp has! You can't call it a Lisp!"

But it spiritually is a Lisp! In Ansible, code is literal YAML data, and literal
YAML data is interpreted as code. This is because Ansible is what happens when
you because you wanted to reduce the amount of code you write, so you write YAML
and pretend it isn't code, but then your YAML becomes so sufficiently complex
that you accidentally horseshoe-theory yourself into Lisp again.

But fine. If you _really_ object to the lack of parens, I can turn it into a
"proper Lisp" for you.

I'm too lazy to write my own json-to-sexp-to-json converter so I'll just use
[this random one I found on the internet](https://github.com/ihalseide/json-sexpr).

```
$ yq < ../playbook.yml > playbook.json
$ python json_sexpr.py playbook.json -s
(list (dict "name" "ansible has syntactic macros ehe :3" "hosts" "::1" "connection" "local" "tasks" (list (dict "name" "Run a shell command and register its output to foo_result" "ansible.builtin.shell" "echo myfoo first" "register" "foo_result") (dict "name" "Syntactically generate some tasks" "copy" (dict "dest" "intermediate.generated.yml" "content" "{{ tasks | to_yaml }}") "vars" (dict "tasks" (list (dict "name" "Print something silly" "ansible.builtin.debug" (dict "msg" "foo_result was templated in as: {{ foo_result.stdout }}"))))) (dict "name" "Set foo_result to something else to test scoping" "ansible.builtin.shell" "echo myfoo second" "register" "foo_result") (dict "name" "Execute the generated tasks" "ansible.builtin.include_tasks" "intermediate.generated.yml"))))
```

Because I love you so much, rhetorical Lisp purist, I even formatted it for you!
I've made Lispible!

```lisp
(list
 (dict "name" "ansible has syntactic macros ehe :3"
       "hosts" "::1"
       "connection" "local"
       "tasks"
       (list (dict "name" "Run a shell command and register its output to foo_result"
                   "ansible.builtin.shell" "echo myfoo first"
                   "register" "foo_result")
             (dict "name" "Syntactically generate some tasks" "copy"
                   (dict "dest" "intermediate.generated.yml"
                         "content" "{{ tasks | to_yaml }}")
                   "vars" (dict
                           "tasks"
                           (list (dict
                                  "name" "Print something silly"
                                  "ansible.builtin.debug" (dict "msg" "foo_result was templated in as: {{ foo_result.stdout }}")))))
             (dict "name" "Set foo_result to something else to test scoping"
                   "ansible.builtin.shell" "echo myfoo second"
                   "register" "foo_result")

             (dict "name" "Execute the generated tasks"
                   "ansible.builtin.include_tasks" "intermediate.generated.yml"))))
```

So now, in order to execute, you just transpile it to YAML like this:

```
$ python json_sexpr.py playbook.sexp -j > playbook.yml
$ ansible-playbook playbook.yml
```

Please don't do this in production.
