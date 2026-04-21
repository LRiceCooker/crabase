import { test, expect } from "./fixtures";

const CONNECTION_STRING = "postgresql://test:test@localhost:5433/crabase_test";

test.describe("Connection flow", () => {
  test("Type connection string, click Next, verify parsed fields", async ({
    page,
  }) => {
    await page.goto("/");

    // Enter connection string
    const input = page.locator(
      'input[placeholder="postgresql://user:password@host:port/dbname"]'
    );
    await input.fill(CONNECTION_STRING);

    // Click Next
    await page.locator('button:has-text("Next")').click();

    // Wait for connection form to appear
    await expect(page.locator('text=Connection details')).toBeVisible();

    // Verify parsed fields
    const hostInput = page.locator('input').nth(0);
    const portInput = page.locator('input').nth(1);
    const userInput = page.locator('input').nth(2);
    const dbInput = page.locator('input').nth(4);

    await expect(hostInput).toHaveValue("localhost");
    await expect(portInput).toHaveValue("5433");
    await expect(userInput).toHaveValue("test");
    await expect(dbInput).toHaveValue("crabase_test");
  });

  test("Click Connect, verify main layout with sidebar tables", async ({
    page,
  }) => {
    await page.goto("/");

    // Quick connect flow
    const input = page.locator(
      'input[placeholder="postgresql://user:password@host:port/dbname"]'
    );
    await input.fill(CONNECTION_STRING);
    await page.locator('button:has-text("Next")').click();
    await expect(page.locator('text=Connection details')).toBeVisible();

    // Click Connect
    await page.locator('button:has-text("Connect")').click();

    // Verify main layout appears with sidebar showing tables
    await expect(page.locator("text=users")).toBeVisible({ timeout: 15000 });
    await expect(page.locator("text=products")).toBeVisible();
    await expect(page.locator("text=events")).toBeVisible();
  });

  test("Verify header shows connection info", async ({ page }) => {
    await page.goto("/");

    // Connect
    const input = page.locator(
      'input[placeholder="postgresql://user:password@host:port/dbname"]'
    );
    await input.fill(CONNECTION_STRING);
    await page.locator('button:has-text("Next")').click();
    await expect(page.locator('text=Connection details')).toBeVisible();
    await page.locator('button:has-text("Connect")').click();

    // Wait for main layout
    await expect(page.locator("text=users")).toBeVisible({ timeout: 15000 });

    // Verify header badges
    await expect(page.locator("text=test@localhost")).toBeVisible();
    await expect(page.locator("text=:5433")).toBeVisible();
    await expect(page.locator("text=crabase_test")).toBeVisible();
  });

  test("Save connection, disconnect, verify saved connection loads", async ({
    page,
  }) => {
    await page.goto("/");

    // Connect with save
    const input = page.locator(
      'input[placeholder="postgresql://user:password@host:port/dbname"]'
    );
    await input.fill(CONNECTION_STRING);
    await page.locator('button:has-text("Next")').click();
    await expect(page.locator('text=Connection details')).toBeVisible();

    // Check "Save connection" checkbox (label not linked via for/id, click checkbox directly)
    await page
      .locator("label:has-text('Save connection')")
      .locator("..")
      .locator('input[type="checkbox"]')
      .click();
    const nameInput = page.locator('input[placeholder="e.g. Production DB"]');
    await nameInput.fill("Test DB");

    // Connect
    await page.locator('button:has-text("Connect")').click();
    await expect(page.locator("text=users")).toBeVisible({ timeout: 15000 });

    // Disconnect
    await page.locator('button[title="Disconnect"]').click();

    // Verify back on connection screen with saved connection visible
    await expect(
      page.locator(
        'input[placeholder="postgresql://user:password@host:port/dbname"]'
      )
    ).toBeVisible();
    await expect(page.locator("text=Test DB")).toBeVisible();

    // Click saved connection to fill the form
    await page.locator("text=Test DB").click();

    // Should navigate to connection form with pre-filled fields
    await expect(page.locator('text=Connection details')).toBeVisible();
  });
});
