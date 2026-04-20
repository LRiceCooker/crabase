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

test.describe("SQL Editor", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Open new SQL editor tab, verify editor visible", async ({ page }) => {
    // Click "+" button to open new SQL editor
    await page.locator('button[title="New SQL Editor"]').click();

    // Verify CodeMirror editor area is visible
    await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 5000 });
  });

  test("Type and run a SELECT query, verify results", async ({ page }) => {
    await page.locator('button[title="New SQL Editor"]').click();
    await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 5000 });

    // Type SQL into CodeMirror
    const editor = page.locator(".cm-content");
    await editor.click();
    await page.keyboard.type("SELECT * FROM users WHERE role = 'admin'");

    // Click Run
    await page.locator('button:has-text("Run")').click();

    // Wait for results table to appear
    await expect(page.locator("table").last()).toBeVisible({ timeout: 10000 });

    // Verify 3 admin rows (alice, diana, heidi)
    await expect(page.locator("text=alice")).toBeVisible();
    await expect(page.locator("text=diana")).toBeVisible();
    await expect(page.locator("text=heidi")).toBeVisible();
  });

  test("Run multi-statement query, verify statement selector", async ({
    page,
  }) => {
    await page.locator('button[title="New SQL Editor"]').click();
    await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 5000 });

    // Type multi-statement SQL
    const editor = page.locator(".cm-content");
    await editor.click();
    await page.keyboard.type(
      "SELECT * FROM users WHERE role = 'admin'; SELECT * FROM products LIMIT 3"
    );

    // Click Run
    await page.locator('button:has-text("Run")').click();

    // Wait for results
    await page.waitForTimeout(2000);

    // Verify statement selector appears (showing multiple statements)
    // The multi-statement navigator shows statement entries
    await expect(page.locator("text=/Statement/i").first()).toBeVisible();
  });

  test("Save query, verify it appears in sidebar", async ({ page }) => {
    await page.locator('button[title="New SQL Editor"]').click();
    await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 5000 });

    // Type SQL
    const editor = page.locator(".cm-content");
    await editor.click();
    await page.keyboard.type("SELECT 1");

    // Save via button
    await page.locator('button:has-text("Save")').click();

    // Wait for save to complete
    await page.waitForTimeout(1000);

    // The query should appear in the sidebar saved queries section
    await expect(page.locator("text=Untitled")).toBeVisible();
  });
});
