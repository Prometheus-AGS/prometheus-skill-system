import { Given, When, Then, setWorldConstructor, World } from '@cucumber/cucumber';
import assert from 'node:assert/strict';

interface AuthWorldFields {
  baseUrl: string;
  email?: string;
  password?: string;
  response?: Response;
  body?: { token?: string; error?: string };
}

class AuthWorld extends World implements AuthWorldFields {
  baseUrl = 'http://localhost:3000';
  email?: string;
  password?: string;
  response?: Response;
  body?: { token?: string; error?: string };
}

setWorldConstructor(AuthWorld);

Given(
  'the auth service is reachable at {string}',
  function (this: AuthWorld, url: string) {
    this.baseUrl = url;
  }
);

Given(
  'a registered user {string} with password {string}',
  function (this: AuthWorld, email: string, password: string) {
    this.email = email;
    this.password = password;
  }
);

When(
  'they POST to {string} with those credentials',
  async function (this: AuthWorld, path: string) {
    this.response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ email: this.email, password: this.password }),
    });
    this.body = (await this.response.json()) as { token?: string; error?: string };
  }
);

When(
  'they POST to {string} with password {string}',
  async function (this: AuthWorld, path: string, password: string) {
    this.response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ email: this.email, password }),
    });
    this.body = (await this.response.json()) as { token?: string; error?: string };
  }
);

When(
  'they POST to {string} with email {string} and password {string}',
  async function (this: AuthWorld, path: string, email: string, password: string) {
    this.response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });
    this.body = (await this.response.json()) as { token?: string; error?: string };
  }
);

Then('the response status is {int}', function (this: AuthWorld, expected: number) {
  assert.equal(this.response?.status, expected);
});

Then(
  'the response body contains a non-empty {string} field',
  function (this: AuthWorld, field: string) {
    const val = (this.body as Record<string, unknown>)?.[field];
    assert.ok(typeof val === 'string' && val.length > 0, `${field} missing or empty`);
  }
);

Then(
  'the token decodes to a subject of {string}',
  function (this: AuthWorld, expectedSub: string) {
    const token = this.body?.token;
    assert.ok(token, 'no token');
    const [, payload] = token.split('.');
    const decoded = JSON.parse(
      Buffer.from(payload, 'base64url').toString('utf8')
    ) as { sub?: string };
    assert.equal(decoded.sub, expectedSub);
  }
);

Then(
  'the response body contains error message {string}',
  function (this: AuthWorld, expected: string) {
    assert.equal(this.body?.error, expected);
  }
);
