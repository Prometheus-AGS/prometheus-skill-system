@ui
Feature: User signs in via the browser
  As a registered user
  I need to sign in through the web UI
  So that I can access my dashboard

  Background:
    Given the app is running at "http://localhost:3000"

  Scenario: Happy path — valid credentials land on dashboard
    Given a registered user "alice@example.com" with password "hunter2"
    When they navigate to the sign-in page
    And they fill the sign-in form with those credentials
    And they submit the form
    Then they land on the dashboard

  Scenario: Invalid password shows an inline error
    Given a registered user "alice@example.com" with password "hunter2"
    When they navigate to the sign-in page
    And they fill the sign-in form with password "wrong"
    And they submit the form
    Then they remain on the sign-in page
    And the form shows the error "Invalid credentials"
