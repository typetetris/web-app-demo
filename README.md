# Web-App-Demo

This is intended to become a little web application demo.

## How to run it

### Using pnpm

1. Clone the repository
1. Start the backend, run `cargo run -p web-app-demo-backend`
   To get more logging, set the environment variable `RUST_LOG` to `info`. On Linux you
   could run `RUST_LOG="info" cargo run -p web-app-demo-backend` for example.
1. Install the frontend dependencies, run `pnpm install`
1. Serve the frontend, run `pnpm -r run dev`
1. Open the browser at the printed url
1. Explore

### Using npm

1. Clone the repository
1. Start the backend, run `cargo run -p web-app-demo-backend`
   To get more logging, set the environment variable `RUST_LOG` to `info`. On Linux you
   could run `RUST_LOG="info" cargo run -p web-app-demo-backend` for example.
1. Install the frontend dependencies, with `./frontend` as the working directory run `npm install`.
   On linux you could do `cd frontend; npm install` for example.
1. Serve the frontend, with `./frontend` as the working directory run `npm dev`.
1. Open the browser at the printed url
1. Explore

## Missing things

There is a lot missing (at the moment):

  - Configurability in the backend like listening endpoint
  - Authentication
  - Authorization
  - Persistence, because of this, I have a memory hog for now, as histories are never expired
  - High Availability
  - Thoroughly checking the app against OWASP Top Ten (and some more maybe)
  - Some functional user stories (all about the notes) aren't implemented yet
  - A CI/CD pipeline
  - A release process
  - Some tests are flaky, because sending messages to a chat via the websocket and then
    requesting a chat history via an endpoint is racy.

## Plan

The app should provide the possiblity to chat with other people and add notes of some kind
to a chat.

A self signup or some kind of administrative UI to manage users are out of scope of
the web application demo.

### User Stories

#### Basic Chatting

- As a user I want to create a new chat room.

- As a user I want to send messages to chat room.

- As a user I want to see messages sent to the chat room.

- As a user I want to invite other people by sending them a link.
  The means of sending the link is not in scope of the demo application.

- As a user I want to be able to open a former chat I participated in and
  see its full history of messages.

#### Basic Note Taking

- As a user I want to be able to create a note in a chat room.

- As a user I want to be able to see all notes in a chat room.

- As a user I want to be able to edit notes which I created.

### Non functional requirements

- The backend should produce useful log messages, so failures can be easily
  investigated.

- Users should be authenticated to ensure their identity.

- User input displayed in the frontend should be properly handled to not
  allow Cross Site Scripting Attachs.

- The OWASP Top Ten security risks are mitigated by following best practices
  from the OWASP Cheat Sheets.

### Development best practices

- CI/CD pipeline should check we don't pull in dependencies with licenses,
  we don't like.

- CI/CD pipeline should check whether there are securiy advisories for any
  of our dependencies.

- CI/CD pipeline should execute the test suite on every build.

### System sketch / Technology choices

We are heading for a classical architecture using a frontend, backend and a database. With some surounding
infrastructure components. For example for authentication we will use a keycloak instance and therefore
OAuth 2.0 and OpenID Connect in some form.

The frontend and backend will communicate by establishing a web socket and maybe additional http requests
for other things.

As a database we will either use postgres, as its more relevant to the audience of this demo
application, or alternatively foundationdb, because I am recently taking a look at it.

#### Frontend

- react
- react-spectrum
- tanstack/query
- react-router

#### Backend

- Rust
- actix-web
- postgresql / foundationdb

We will use a broadcast per chat, so every participant has a web socket connection listening to that broadcast,
sending a message to a chat means sending the message to the broadcast.

#### Authentication

- keycloak

### Implementation Steps

1. Functional part of the backend to provide some basic chatting users stories. For now without persistence and authentication.
   But with logging for better debugging and developer experience.

   1. Have a skeleton ready - ✅
   1. Introduce ChatServer abstraction - ✅
   1. Fix failing test - ✅
   1. Add some abstractions for recurring test tasks - ✅
   1. Add a test for multiple chats, users - ✅
   1. Fix the concurrency issue for the dashmaps - ✅
   1. Fix the `ChatServer` naming and introduce `ChatServerHandles` to make ownership/behaviour of clone clearer. - ❌
   1. Remove `Clone` implementation from `ChatServer` - ✅
   1. Add a test for the broadcast cleanup - ✅
   1. Split up the chat module as it gets rather unwieldy now. - ✅
   1. Add an endpoints to - ✅
      - get chat histories - ✅
      - create chats - ❌
      - join a chat (via websockts) - ✅

2. Functional part of the frontend to allow interfacing the backend created in step 1. to quickly have a demostrable product. - ✅
  1. Add all the missing error and pending views with proper react-spectrum components.

3. Add authentication
    1. Add keycloak infrastructure component.
    2. Add keycloak client library to the frontend and do the login dance with keycloak providing access tokens to the backend.
    3. Check access tokens in the backend, for now only on each http request.
       Warning: The long lived web socket connection remains a security risk yet, as it might well exist past the expiration time of the token used to initiate it.

4. Decide which kind of persistence to add (postgresql/foundationdb) and do it.

5. Implement some more user stories.

6. Go through OWASP cheat sheets and see, what we still need to implement. For example rate limiting.
