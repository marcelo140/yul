Rc and RefCell are indeed necessary in the case of Env. The runtime can mutate Env through *def!* and both Lambda and other inner Env need to be able to read it at the same time. Thus, the inherent ownership model does not work here since it forces you to choose between an unique mutable reference or shared references.

I would like that lisp functions were also part of the rust type system. Right now everything is encapsulated inside a big MValue.

Accessing the list of parameters in eval, eval_ast and the core fucntions seems really dirty.
