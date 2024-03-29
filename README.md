# cs262

**Status: Complete**

These are my solutions to [CS 262: Introduction to Distributed Computing](https://canvas.harvard.edu/courses/116261) assignments at Harvard, taught by [Jim Waldo](http://www.eecs.harvard.edu/~waldo/) in Spring 2023.

I'm not taking this class, so this is just for fun. I'm writing my solutions in Rust; run with `cargo run`.

I may skip parts of assignments that involve a lot of "writeup" work. Generally I don't want to spend more than 1-2 hours per assignment, so the code might be written quite quickly.

## Assignment 1: Wire Protocols

![](screenshots/wire.png)

This is located in the [`wire`](src/wire.rs) module. It just contains a simple but compact wire protocol and uses it to send chat messages over TCP.

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

## Assignment 2: Scale Models and Logical Clocks

![](screenshots/lamport.png)

This is located in the [`lamport`](src/lamport.rs) module. The simulation uses threads communicating over shared MPSC channels.

> In this assignment, you and your partner will build a model of a small, asynchronous distributed system. It will run on a single machine, but you will model multiple machines running at different speeds. And you will build a logical clock for each of the model machines.
>
> Each model machine will run at a clock rate determined during initialization. You will pick a random number between 1 and 6, and that will be the number of clock ticks per (real world) second for that machine. This means that only that many instructions can be performed by the machine during that time. Each machine will also have a network queue (which is not constrained to the n operations per second) in which it will hold incoming messages. The (virtual) machine should listen on one or more sockets for such messages.
>
> Each of your virtual machines should connect to each of the other virtual machines so that messages can be passed between them. Doing this is part of initialization, and not constrained to happen at the speed of the internal model clocks. Each virtual machine should also open a file as a log. Finally, each machine should have a logical clock, which should be updated using the rules for logical clocks.
>
> Once initialization is complete, each virtual machine should work according to the following specification:
>
> On each clock cycle, if there is a message in the message queue for the machine (remember, the queue is not running at the same cycle speed) the virtual machine should take one message off the queue, update the local logical clock, and write in the log that it received a message, the global time (gotten from the system), the length of the message queue, and the logical clock time.
>
> If there is no message in the queue, the virtual machine should generate a random number in the range of 1-10, and
>
> - if the value is 1, send to one of the other machines a message that is the local logical clock time, update it’s own logical clock, and update the log with the send, the system time, and the logical clock time
> - if the value is 2, send to the other virtual machine a message that is the local logical clock time, update it’s own logical clock, and update the log with the send, the system time, and the logical clock time.
> - if the value is 3, send to both of the other virtual machines a message that is the logical clock time, update it’s own logical clock, and update the log with the send, the system time, and the logical clock time.
> - if the value is other than 1-3, treat the cycle as an internal event; update the local logical clock, and log the internal event, the system time, and the logical clock value.
>
> While working on this, keep a lab notebook in which you note the design decisions you have made. Then, run the scale model at least 5 times for at least one minute each time. Examine the logs, and discuss (in the lab book) the size of the jumps in the values for the logical clocks, drift in the values of the local logical clocks in the different machines (you can get a god’s eye view because of the system time), and the impact different timings on such things as gaps in the logical clock values and length of the message queue. Observations and reflections about the model and the results of running the model are more than welcome.

## Assignment 3: Replication

![](screenshots/wire2.png)

This is located in the [`wire2`](src/wire2.rs) module. It extends the chat server with multi-process SQLite for a persistent message store, and gains fault tolerance simply by binding multiple servers to the same local port with `SO_REUSEADDR`.

If desired to run the server on multiple nodes, it would need some kind of consensus. You could [abuse NFS for this](https://www.sqlite.org/useovernet.html), or pick up an over-the-counter solution for SQLite like [rqlite](https://github.com/rqlite/rqlite). A more boring choice would be to use a replicated client-server database like PostgreSQL or Redis. It's a bit unclear whether this falls within the intended scope of the assignment, but you could also implement a distributed K/V store from scratch, using Raft for instance (like a very simple [TiKV](https://tikv.org/)).

> Take one of the two implementations you created for the first design exercise (the chat application) and re-design it so that the system is both persistent (it can be stopped and re-started without losing messages that were sent during the time it was running) and 2-fault tolerant in the face of crash/failstop failures. In other words, replicate the back end of the implementation, and make the message store persistent.
>
> The replication can be done in multiple processes on the same machine, but you need to show that the replication also works over multiple machines (at least two). That should be part of the demo.
>
> As usual, you will demo the system on Demo Day III (April 10). Part of the assignment is figuring out how you will demo both the new features. As in the past, keep an engineering notebook that details the design and implementation decisions that you make while implementing the system.

## License

All code is licensed under the [MIT license](LICENSE).
