## Timesheets

- Users are registered to be able to sign their timesheet
- A user would have details such as employee number, name, address and hourly rate
- They would 'enter' and at that point they would have a sign in time.  
- They can only 'exit' if they have entered.
- There would be a maximum time one can be in the state 'enter' after which their
state would be set back to 'exit'.  This would be configurable, with a default of 8.
- After each day a payroll would be scheduled to credit those that have worked based on an hourly rate.
- The payroll would be funded with the treasury, maybe...
- We have an admin that adds a user

Why - combined with IoT and remote woring it would allow trustable and flexible work.  It would allow for
payment to be sent on a daily basis.

### Events

UserRegistered(user_id: u32, user: User)
UserEntered(user_id, user: User, time: Time)
UserExited(user_id, user: User, time: Time)
UserPaid(user_id)

### Errors

InvalidUser
FailedPayment
FailedRegister
FailedToEnter
FailedToExit

### Calls

/// Register a user, the origin should be root
fn register_user(origin, user: User)
/// Enter as a user, fails if we aren't registered or we are already entered
fn enter_user(origin)
/// Exit as a user, fails if we aren't registered or we are already exited
fn exit_user(origin)
/// Update rate of user, this should be multisigned by both admin and user
fn update_rate_user(origin, user_id, rate: Rate)

### Other

We would on X interval of time pay the set amount to the user

## Vacation

- A registered user would have an allocated number of vacation hours
- Vacation would be booked at intervals of set minimum which is configurable
- Vacation would be approved from a set minimum amount
- The user would be able to cancel booked holiday and the allowance would be returned
- The user wouldn't be able to cancel a booked holiday during or after the event
- An amount of redeemable holidays would be set for those holidays not enjoyed and credited at the end of the year
