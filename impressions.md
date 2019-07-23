Things are getting so nasty that I preferred to add Rc everywhere to allow for clones and avoid the type system. Could lifetimes solve the problem? Lifetimes are like cancer and would spread to every other part of the code probably, which is pain. The thing is that I really didn't explore if I really need Rc.

I would like that lisp functions were also part of the rust type system. Right now everything is encapsulated inside a big MValue.

Accessing the list of parameters in eval, eval_ast and the core fucntions seems really dirty.
