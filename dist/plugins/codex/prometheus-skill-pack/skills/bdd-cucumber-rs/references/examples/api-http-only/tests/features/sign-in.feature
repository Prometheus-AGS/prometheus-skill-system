@api
Feature: User signs in via the auth API
  As a client of the auth service
  I need to exchange credentials for a session token
  So that I can call authenticated endpoints

  Background:
    Given the auth service is reachable at "http://localhost:3000"

  Scenario: Valid credentials return a token
    Given a registered user "alice@example.com" with password "hunter2"
    When they POST to "/api/auth/sign-in" with those credentials
    Then the response status is 200
    And the response body contains a non-empty "token" field

  Scenario: Invalid password returns 401
    Given a registered user "alice@example.com" with password "hunter2"
    When they POST to "/api/auth/sign-in" with password "wrong"
    Then the response status is 401
    And the response body contains error message "invalid credentials"
