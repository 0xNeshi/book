## Concurrency With Async

In this section, we will apply async to some of the same concurrency challenges
we tackled with threads in chapter 16. Since we already talked about a lot of
the key ideas there, in this section we will focus on what is different between
threads and futures.

In many cases, the APIs for working with concurrency using async are very
similar to those for using threads. In other cases, they end up being shaped
fairly differently. Even when the APIs look similar, they often have different
behavior and they nearly always have different performance characteristics.

### Counting

The first task we tackled in Chapter 16 was counting up on two separate threads.
Let’s do the same using async. The `trpl` crate supplies a `spawn_task` function
which looks very similar to the `thread::spawn` API, and a `sleep` function
which is an async version of the `thread::sleep` API. We can use these together
to implement the same counting example as with threads.

Listing 17-4 shows our starting point. We set up our `main` function with `trpl::block_on`, so that our top-level function can be async.

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-04/src/main.rs:block_on}}
```

> Note: From this point forward in the chapter, every example will include this
> exact same wrapping code with `trpl::block_on` in `main`, so we will often
> skip it just like we do with `main`. Don’t forget to include it in your
> code!

Then we can write two loops within that block, each with a `trpl::sleep` call in
them. Similar to the threading example, we put one loop in the body of a
`trpl::spawn_task`, the same way we did with `thread::spawn`, and the other in a
top-level `for` loop. Notice that we also need to add a `.await` after the
`sleep` calls.

<Listing number="17-5" caption="Using `spawn_task` to count with two" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-05/src/main.rs:task}}
```

</Listing>

This does something very similar to what the thread-based implementation did, as
we can see from the output when we run it. (As with the threading example, you
may see a different order in your own terminal output when you run this.)

<!-- Not extracting output because changes to this output aren't significant;
the changes are likely to be due to the threads running differently rather than
changes in the compiler -->

```text
hi number 1 from the second task!
hi number 1 from the first task!
hi number 2 from the first task!
hi number 2 from the second task!
hi number 3 from the first task!
hi number 3 from the second task!
hi number 4 from the first task!
hi number 4 from the second task!
hi number 5 from the first task!
```

This stops as soon as the for loop in the body of the main async block finishes,
because the task spawned by `spawn_task` is shut down when the main function
ends—just like threads are. Thus, if you want to run all the way to the
completion of the task, you will need to use a join handle to wait for the first
task to complete. With threads, we used the `join` method to “block” until the
thread was done running. Here, we can use `await` to do the same thing, and
since the handle’s output is actually a `Result`, we will also unwrap it.

<Listing number="17-6" caption="Using `.await` with a join handle to run a task to completion" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-06/src/main.rs:handle}}
```

</Listing>

Now the output again looks like what we saw in the threading example. (Again,
the exact output may look different for you.)

<!-- Not extracting output because changes to this output aren't significant;
the changes are likely to be due to the threads running differently rather than
changes in the compiler -->

```text
hi number 1 from the second task!
hi number 1 from the first task!
hi number 2 from the first task!
hi number 2 from the second task!
hi number 3 from the first task!
hi number 3 from the second task!
hi number 4 from the first task!
hi number 4 from the second task!
hi number 5 from the first task!
hi number 6 from the first task!
hi number 7 from the first task!
hi number 8 from the first task!
hi number 9 from the first task!
```

So far, it looks like async and threads basically give us the same basic
behavior. However, there are a few important differences already. One was using
`.await` instead of calling `join` on the join handle. Another is that we needed
to await both `sleep` calls. Most importantly, though, we did not need to spawn
another operating system thread to do this. We were able to get concurrency for
just the cost of a task, which has much faster startup time and uses much less
memory than an OS thread.

What is more, we actually do not need the `spawn_task` call at all to get
concurrency here. Remember that each async block compiles to an anonymous
future. That means we can put each of these two loops in an async block and then
ask the runtime to run them both to completion using `trpl::join`:

<Listing number="17-7" caption="Using `trpl::join` to await two anonymous futures" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-07/src/main.rs:join}}
```

</Listing>

When we run this, we see both futures run to completion:

<!-- Not extracting output because changes to this output aren't significant;
the changes are likely to be due to the threads running differently rather than
changes in the compiler -->

```text
hi number 1 from the first task!
hi number 1 from the second task!
hi number 2 from the first task!
hi number 2 from the second task!
hi number 3 from the first task!
hi number 3 from the second task!
hi number 4 from the first task!
hi number 4 from the second task!
hi number 5 from the first task!
hi number 6 from the first task!
hi number 7 from the first task!
hi number 8 from the first task!
hi number 9 from the first task!
```

Here, you will see the exact same order every time, which is very different from
what we saw with threads. That is because the `trpl::join` function is *fair*,
meaning it checks both futures equally, rather than letting one race ahead. With
threads, the operating system decides which thread to check, and that is
ultimately out of our control. With an async runtime, the runtime itself decides
which future to check, so it has the final say. In practice, the details get
complicated because an async runtime might use operating system threads under
the hood as part of how it manages concurrency, but a runtime can still choose
to guarantee fairness even so. However, runtimes do not have to guarantee
fairness for any given operation, and even within a given runtime, different
APIs sometimes exist to let you choose whether fairness is something you care
about as a caller.

Try some of these different variations on awaiting the futures and see what they
do:

* Remove the async block from around either or both of the loops.
* Await each async block immediately after defining it.
* Wrap only the first loop in an async block, and await the resulting future
  after the body of second loop.

For an extra challenge, see if you can figure out what the output will be in
each case *before* running the code!

### Message Passing

Sharing data between futures will look familiar. We can again use async versions
of Rust’s types for message-passing. Instead of `std::sync:mpsc::channel`, we
will use a `tprl::channel`, for example.

The synchronous `Receiver::recv()` method in `std::mpsc::channel` blocks until
it receives a message. The `trpl::Receiver::recv()` method, by contrast, is an
`async` function. Instead of blocking, it waits until a message is received or
the send side of the channel closes. One other difference with this particular
`recv()` implementation is that it returns an `Option` of the type sent over the
channel instead of a `Result`.

We can start by introducing an async version of the multiple-producer,
single-consumer channel channel API we used with threads back in Chapter 16. The
API is just a little different here in Listing 17-8: we have a mutable receiver
`rx`. Otherwise, this looks pretty much the same as the thread-based approach.

<Listing number="17-8" caption="Creating an async channel and assigning the two halves to `tx` and `rx`" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-08/src/main.rs:add-channel}}
```

</Listing>

Now we can send messages from the sender to the receiver. Again, the API is just
a little different from the threaded version in Chapter 16, where we needed to
spawn a separate thread to allow the message passing to happen asynchronously.
In the version in Listing 17-9, we opt into async behavior on the receiver side
by using `.await` on the `rx.recv()` call.

<Listing number="17-9" caption='Sending `"hi"` from `tx` and receiving it in `rx`' file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-09/src/main.rs:send-and-receive}}
```

</Listing>

The `send` call does not block, since the channel we are sending it into is
unbounded. That was true with our threading example back in Chapter 16, too,
though. However, there is a big difference with the `rx.recv()` calls. The one
back in Chapter 16 blocked the thread it ran on—in that case, the main thread.
This one does not block at all! Instead, once the program hits the `.await` on
the `rx.recv()` call, it hands control back to the runtime, which can go on
scheduling other operations until a message arrives. It might be hard to see
that from this code, though, since the message will arrive right away!

> Note: Since this is all wrapped in a `trpl::block_on`, this would effectively
> block anything happening outside that. That is the whole point of `block_on`,
> in fact: to allow you to *choose* where to block on some set of async code to
> transition between sync and async code. However, *within* this block, the
> `.await` does not block further operations—as we will see!

Let’s go ahead and send a whole series of messages, and sleep in between them,
as shown in Listing 17-10:

<Listing number="17-10" caption="Sending multiple messages over the async channel and sleeping with an `.await` between each message" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-10/src/main.rs:many-messages}}
```

</Listing>

This handles sending the messages, but so far we don’t do anything with them,
and the code just silently runs forever. We need to actually *receive* the
messages. In this case, we could do that manually, because we know how many
messages are coming in. In the real world, though, we will generally be waiting
on some *unknown* number of messages. In that case, we need to keep waiting
until we determine that there are no more messages.

That sounds like a good job for a loop! In synchronous code, we might use a
`for` loop to process a sequence of items, regardless of how many items are in
the loop. However, Rust does not yet have a way to write a `for` loop over an
*asynchronous* series of items. Instead, we need to use a new kind of loop we
haven’t seen before, the `while let` conditional loop. A `while let` loop is the
loop version of the `if let` construct we saw back in Chapter 6. It continues as
long as the condition it relies on is true. Listing 17-11 shows how we can use
this with `rx.recv` to print all the messages send by the `tx` transmitter.

<!-- TODO: update text in ch. 19 to account for our having introduced this. -->

<Listing number="17-11" caption="Using a `while let` loop with `.await` to receive messages asynchronously" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-11/src/main.rs:loop}}
```

</Listing>

The `rx.recv()` call produces a `Future`. The `Output` of the future is an
`Option` of the message type. While waiting on messages, it will respond with
`Poll::Pending`, so the runtime will pause it until it is time to check it
again. Once a message arrives, it will respond with
`Poll::Ready(Some(message))`. When the channel closes, it will instead respond
with `Poll::Ready(None)`, which we can use to know that it is done. The `while
let` pulls all of this together. If the result of calling `rx.recv().await` is
`Some(message)`, we get access to the message and we can use it in the loop
body, just like we could with `if let`. If the result is `None`, the loop ends.
Every time the loop completes, it hits the await point again, so the runtime
pauses it again until another message arrives.

With the `while let` loop in place, the code now successfully sends and receives
the messages. Unfortunately, there are still a couple problems. For one thing,
the messages do not arrive at one-second intervals, we see them arrive all at
once, four seconds after we start the program. For another, this program also
never stops! You will need to shut it down using <span
class="keystroke">ctrl-c</span>.

Let’s start by understanding why the messages all come in at once after the full
delay, rather than coming in with delays in between each one. This highlights an
important point about the way that async works in Rust. Within any given async
block, the await points are sequential: each one happens one after another. That
is, after all, one of the big motivations for using this syntax instead of
callbacks, event handlers, or chains of methods: the flow through the program is
much easier to follow, because having the order that `.await` keywords appear in
the *code* is also the order they happen when running the *program*.

With that in mind, we can see why this code behaves the way it does by looking
at the whole thing all together, in Listing 17-12.

<Listing number="17-12" caption="An async block with multiple `.await` points in it" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-12/src/main.rs:all}}
```

</Listing>

There is just one async block here, so everything here will proceed linearly.
Every one of the `.await` points for the `trpl::sleep` calls appears before the
`.await` points on the `rx.recv()`, so all the `tx.send` calls happen,
interspersed with all of the `trpl::sleep` calls. Only then does the `while let`
loop get to go through all of the `.await` points on the `recv` calls.

To get the behavior we actually want, where the delay happens in between
receiving each message, rather than before receiving any message, we need to
give put the `tx` and `rx` operations in their own async blocks, so the runtime
can execute each of them separately. We also need to tell the runtime to
actually run them using `trpl::join`, just like we did for the counting example
above. Listing 17-13 shows how that looks.

<Listing number="17-13" caption="Separating `send` and `recv` into their own `async` blocks and awaiting the futures for those blocks" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-13/src/main.rs:futures}}
```

</Listing>

With these changes made, the messages get printed at one-second intervals,
rather than all in a rush after four seconds.

The program still never stops running, though. That’s because of the combination
of the `while let` loop and the `trpl::join` call. Let’s consider the way this
loop works:

* The `trpl::join` future only completes once *both* futures passed to it
  have completed.
* The `tx` future completes once it finishes sleeping after sending the last
  message in `vals`.
* The `rx` future will not complete until the `while let` loop ends.
* The `while let` loop will not end until `rx.recv().await` produces `None`.
* The `rx.recv().await` will only return `None` once the other end of the
  channel is closed.
* The channel will only close if we call `rx.close()` or when the sender side,
  `tx`, is dropped.
* We do not call `rx.close()` anywhere, and `tx` will not be dropped until the
  async block ends.
* The block cannot end because it is blocked on `trpl::join` completing,
  which takes us back to the top of this list!

We need to make sure the channel gets closed so that `trpl::join` will complete.
We could manually close `rx` somewhere by calling `rx.close()`, but that does
not make much sense in this case. The idea is that `rx` should keep listening
until `tx` is done sending. Stopping after handling some arbitrary number of
messages would make the program shut down, but it would mean we could miss
messages if the sending side changed. Given that we cannot use `rx.close()`, we
need to make sure that `tx` gets dropped *before* the end of the function.

Right now, the async block only borrows `tx`. We can confirm this by adding
another async block which uses `tx`, and using `trpl::join3` to wait for all
three futures to complete:

<Listing number="17-14" caption="Adding another async block which borrows `tx`, to see that we can borrow it repeatedly" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-14/src/main.rs:updated}}
```

</Listing>

Now both blocks borrow `tx`, so they are both able to use it to send messages,
which `rx` can then receive. When we run that code, we see the extra output from
the new `async` block, and the message it sends being received by the
`rx.recv()`.

<!-- Not extracting output because changes to this output aren't significant;
the changes are likely to be due to the threads running differently rather than
changes in the compiler -->

```text
Got: hi
Got: more
Got: from
Got: messages
Got: the
Got: for
Got: future
Got: you
```

As before, we also see that the program does not shut down on its own and
requires a <span class="keystroke">ctrl-c</span>. This little exploration helps
us understand why: it is ultimately about *ownership*. We need to move `tx` into
the async block so that once that block ends, `tx` will be dropped.

Since we have seen how `async` blocks borrow the items they reference from their
outer scope, we can go ahead and remove the extra block we just added for now,
and switch back from `join3` to `join`.

The last step here is to figure out how to get ownership of the data instead of
just borrowing it. In Chapter 13, we learned how to use the `move` keyword with
closures, and in Chapter 16, we saw that we often need to use closures marked
with `move` when working with threads. As we have discovered, the same dynamics
apply to async blocks! Hopefully this will make sense if you remember that any
time you write a future, a runtime is ultimately responsible for executing it.
That means that an async block might outlive the function where you write it,
the same way a closure can. When a future takes ownership of the data it
references this way, it needs to move that data into the future—so the `move`
keyword works with async blocks just like it does with closures.

Thus, we can change the first async block from an `async` block to an `async
move` block, like this:

The result is Listing 17-15, and when we run *this* version of the code, it
shuts down gracefully after the last message is sent.

<Listing number="17-15" caption="A working example of sending and receiving messages between futures which correctly shuts down when complete" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-15/src/main.rs:with-move}}
```

</Listing>

This async channel is also a multiple-producer channel, so we can call `clone`
on `tx` if we want to send messages from multiple futures. For example, we can
make the code from Listing 17-16 work by cloning the `tx` before moving it
into the first async block, moving the original `tx` into the second async
block, and switching back to `join3`.

<Listing number="17-16" caption="Using multiple producers with async blocks" file-name="src/main.rs">

```rust
{{#rustdoc_include ../listings/ch17-async-await/listing-17-16/src/main.rs:here}}
```

</Listing>

*Both* of these blocks need to be `async move` blocks, or else we will end up
back in the same infinite loop we started out in. With that done, though, we get
all the messages we expected, with little delays between them. Notice that since
each of the sending futures do a one-second delay after sending, the messages
come in right after each other at one-second intervals. The delays are
concurrent, not sequential, just as we would expect.

This is a good start, but it limits us to just a handful of futures: two with
`join`, or three with `join3`. Let’s see how we might work with more futures.

[streams]: /ch17-05-streams.md
