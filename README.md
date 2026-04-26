# Vapour

The software for a payment kiosk allowing college students to topup college payment accounts to pay for things in college.

Designed specifically in mind for Runshaw College, but with some api changes can be used for anything.

## Why was this made?

Our college (Runshaw) uses a custom payment system internally instead of relying on bank cards and cash. There are two terminals that allow you to do this in the college, however their software is buggy, has poor accessibility and also can be escaped by users.

## Software Goals

- Provide a less buggy and consistent experience
- Add accessibility options for users with impaired visuals (i.e: high contrast mode)
- Attempt to reduce escaping. (May also be more for the operating system's configurations)

## Non-goals (But would like)

- Provide a cleaner and more pleasant user interface with animations. Currently, there is a lot of unused screen real estate.
- Provide a way for other colleges to implement their own APIs and use this internally

## How to run

```bash
cargo run
```
