import { createBdd } from 'playwright-bdd';
import { expect } from '@playwright/test';

const { Given, When, Then } = createBdd();

Given('the app is running at {string}', async ({}, url: string) => {
  // Base URL configured in playwright.config.ts — this step documents the
  // dependency for readers; nothing to do at runtime.
  expect(url).toMatch(/^https?:\/\//);
});

Given(
  'a registered user {string} with password {string}',
  async ({ page }, email: string, password: string) => {
    await page.context().addInitScript(
      (u) => ((window as unknown as Record<string, unknown>).__testUser = u),
      { email, password }
    );
  }
);

When('they navigate to the sign-in page', async ({ page }) => {
  await page.goto('/sign-in');
  await expect(page.getByTestId('sign-in-form')).toBeVisible();
});

When(
  'they fill the sign-in form with those credentials',
  async ({ page }) => {
    const testUser = await page.evaluate(
      () => (window as unknown as { __testUser: { email: string; password: string } }).__testUser
    );
    await page.getByTestId('email-input').fill(testUser.email);
    await page.getByTestId('password-input').fill(testUser.password);
  }
);

When(
  'they fill the sign-in form with password {string}',
  async ({ page }, password: string) => {
    const testUser = await page.evaluate(
      () => (window as unknown as { __testUser: { email: string } }).__testUser
    );
    await page.getByTestId('email-input').fill(testUser.email);
    await page.getByTestId('password-input').fill(password);
  }
);

When('they submit the form', async ({ page }) => {
  await page.getByTestId('submit-button').click();
});

Then('they land on the dashboard', async ({ page }) => {
  await expect(page).toHaveURL(/\/dashboard/);
});

Then(
  'the header greets them by name {string}',
  async ({ page }, name: string) => {
    await expect(page.getByTestId('user-greeting')).toContainText(name);
  }
);

Then('they remain on the sign-in page', async ({ page }) => {
  await expect(page).toHaveURL(/\/sign-in/);
});

Then('the form shows the error {string}', async ({ page }, msg: string) => {
  await expect(page.getByTestId('form-error')).toHaveText(msg);
});

Then('the email input is marked invalid', async ({ page }) => {
  await expect(page.getByTestId('email-input')).toHaveAttribute(
    'aria-invalid',
    'true'
  );
});

Then('the password input is marked invalid', async ({ page }) => {
  await expect(page.getByTestId('password-input')).toHaveAttribute(
    'aria-invalid',
    'true'
  );
});
