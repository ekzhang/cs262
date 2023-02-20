# cs262

These are my assignment solutions to [CS 262: Introduction to Distributed Computing](https://canvas.harvard.edu/courses/116261) at Harvard, taught by [Jim Waldo](http://www.eecs.harvard.edu/~waldo/) in Spring 2023.

I'm not taking this class, so this is mostly just for fun. I'm doing this in Rust.

I may skip parts of assignments that I don't find interesting. Generally I don't want to spend more than 1-2 hours per assignment, so you shouldn't expect production-quality code.

## Assignment 1: Wire Protocols

This is located in the `wire` module.

> For the first design exercise, you will develop a simple chat application. This will be a client/server application, with the following functions:
>
> 1. Create an account. You must supply a unique user name.
> 2. List accounts (or a subset of the accounts, by text wildcard)
> 3. Send a message to a recipient. If the recipient is logged in, deliver immediately; otherwise queue the message and deliver on demand. If the message is sent to someone who isn't a user, return an error message
> 4. Deliver undelivered messages to a particular user
> 5. Delete an account. You will need to specify the semantics of what happens if you attempt to delete an account that contains undelivered message.
>
> The purpose of this assignment is to get you to design a wire protocol. So the solution is not to go looking for a library that will do this work for you. You should use sockets and transfer buffers (of your definition) between the machines.
>
> You will need to write a specification of the wire protocol used in the system, and then build a client and server that use that protocol. It should be possible for multiple clients to connect to the server at the same time; you can assume a single instance of the server is all that is needed at this point.

## License

All code is licensed under the [MIT license](LICENSE).
