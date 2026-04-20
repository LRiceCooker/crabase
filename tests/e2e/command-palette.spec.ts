import { test, expect } from "@playwright/test";

const CONNECTION_STRING = "postgresql://test:test@localhost:5433/crabase_test";

async function connectToDb(page) {
  await page.goto("/");
  const input = page.locator(
    'input[placeholder="postgresql://user:password@host:port/dbname"]'
  );
  await input.fill(CONNECTION_STRING);
  await page.locator('button:has-text("Next")').click();
  await expect(page.locator("text=Connection details")).toBeVisible();
  await page.locator('button:has-text("Connect")').click();
  await expect(page.locator("text=users")).toBeVisible({ timeout: 15000 });
}

test.describe("Command palette", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Cmd+Shift+P opens command palette with focused input", async ({
    page,
  }) => {
    await page.keyboard.press("Meta+Shift+P");

    // Verify command palette is visible
    const input = page.locator('input[placeholder="Type a command..."]');
    await expect(input).toBeVisible();
    await expect(input).toBeFocused();
  });

  test("Type to filter commands", async ({ page }) => {
    await page.keyboard.press("Meta+Shift+P");

    const input = page.locator('input[placeholder="Type a command..."]');
    await input.fill("Restore");

    // Verify "Restore Backup" appears in results
    await expect(page.locator("text=Restore Backup")).toBeVisible();
  });

  test("Escape closes command palette", async ({ page }) => {
    await page.keyboard.press("Meta+Shift+P");
    await expect(
      page.locator('input[placeholder="Type a command..."]')
    ).toBeVisible();

    await page.keyboard.press("Escape");

    // Verify palette is closed
    await expect(
      page.locator('input[placeholder="Type a command..."]')
    ).not.toBeVisible();
  });

  test("Cmd+P opens table finder with tables listed", async ({ page }) => {
    await page.keyboard.press("Meta+P");

    // Verify table finder is visible
    const input = page.locator(
      'input[placeholder="Search tables and queries..."]'
    );
    await expect(input).toBeVisible();

    // Verify tables are listed
    await expect(page.locator("text=users")).toBeVisible();
    await expect(page.locator("text=products")).toBeVisible();
  });

  test("Table finder: type to filter, Enter opens table", async ({ page }) => {
    await page.keyboard.press("Meta+P");

    const input = page.locator(
      'input[placeholder="Search tables and queries..."]'
    );
    await input.fill("user");

    // Verify "users" is filtered
    await expect(page.locator("text=users")).toBeVisible();

    // Press Enter to open
    await page.keyboard.press("Enter");

    // Verify users table tab opens
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });
  });

  test("Opening Cmd+P closes Cmd+Shift+P (no stuck state)", async ({
    page,
  }) => {
    // Open command palette
    await page.keyboard.press("Meta+Shift+P");
    await expect(
      page.locator('input[placeholder="Type a command..."]')
    ).toBeVisible();

    // Open table finder
    await page.keyboard.press("Meta+P");

    // Command palette should be closed
    await expect(
      page.locator('input[placeholder="Type a command..."]')
    ).not.toBeVisible();

    // Table finder should be open
    await expect(
      page.locator('input[placeholder="Search tables and queries..."]')
    ).toBeVisible();
  });
});
