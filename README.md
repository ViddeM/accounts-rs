# AccountsRS - An accounts service

A simple account management service that can/will be used together with other **vmcorp** projects, e.g. [vrecipes](github.com/viddem/vrecipes).

## Goals
 1. Store & manage basic account information:
    - Name
    - Username
    - Password
    - Email
 1. Have a simple CRUD on top of this data using a simple REST api
 1. Implement oauth2 provider support to authenticate users towards other services. (Maybe done separately?)
 1. Allow creation of accounts using google accounts and similar providers.
