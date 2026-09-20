# Architecture Notes — Why This Slice Exists

## 1. The tty7 lesson

The important lesson from tty7 is not:

    "use a daemon"

and not:

    "render more things on the GPU"

It is:

    state authority
        !=
    presentation session
        !=
    rendered surface

tty7 can make its GUI disposable because its authoritative PTY state lives
elsewhere.

Shelly does not need that lifecycle model.

But Shelly GPUI does need the same separation of responsibility.

---

## 2. Shelly-specific mapping

tty7:

    PTY daemon
        -> workspace/session state
        -> terminal surfaces

Shelly GPUI:

    Shelly backend
        -> PackageStore / AppSession
        -> package surfaces

The session layer may retain:

- search results;
- selected package identity;
- detail data;
- update state;
- navigation state;

without pretending to be authoritative package state.

The Shelly backend remains authoritative.

---

## 3. Why Entity<T>

Current GPUI documentation explicitly models stateful application objects as
entities.

An entity can be updated independently and can notify observers.

This is preferable to keeping all state inside one large render root because it
creates explicit ownership and invalidation boundaries.

Conceptually:

    PackageStore.update(...)
        -> cx.notify()

should invalidate consumers of PackageStore.

A pane-width pointer update should not semantically be a PackageStore update.

A search query update should not semantically be a console update.

This distinction becomes architectural rather than conventional.

---

## 4. observe vs subscribe

Prefer:

    cx.observe()

when a view/entity needs to recompute because another entity changed.

Prefer:

    cx.subscribe() + cx.emit()

for semantic events.

Example:

PackageStore changing package results:

    observe

Package mutation completed:

    emit MutationCompleted(PackageKey)

A store can subscribe and invalidate affected records.

Avoid creating event traffic merely to imitate function calls.

---

## 5. Virtualization remains presentation

`uniform_list` solves visible-item construction.

It does not solve:

- backend request scheduling;
- session caching;
- package identity;
- state invalidation;
- search races.

Do not confuse list virtualization with application-state architecture.

---

## 6. Animation is deliberately deferred

Current GPUI exposes frame scheduling and animation APIs.

That does not mean this slice should immediately add page transitions.

Animation should be introduced only after:

- navigation state is explicit;
- surfaces are independently owned;
- transitions have stable source/destination states.

Otherwise animation code will become entangled with the same monolithic state
that this slice is intended to remove.

---

## 7. The future motion model

For future reference only, eventual motion should support:

- duration;
- easing;
- interruption;
- direction reversal;
- reduced motion;
- state-driven completion.

Likely GPUI building blocks include:

    with_animation
    with_animations
    request_animation_frame
    on_next_frame

Do not implement this in Slice-02.
