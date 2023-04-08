# AccountsRS - An accounts service

A simple account management service that can/will be used together with other **vmcorp** projects, e.g. [vrecipes](github.com/viddem/vrecipes).

## Goals
 1. Store & manage basic account information:
    - Name
    - Username
    - Password
    - Email
 2. Have a simple CRUD on top of this data using a simple REST api
 3. Implement oauth2 provider support to authenticate users towards other services. (Maybe done separately?)
 4. Allow creation of accounts using google accounts and similar providers.

## TODO
 1. Track geolocation and require email confirmation upon logging in from a new location
 2. Implement TOTP (one-time authenticator password), maybe use totp-rs
