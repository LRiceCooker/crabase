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

test.describe("Context menus", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Right-click table in sidebar shows context menu", async ({ page }) => {
    // Right-click on "users" in the sidebar
    await page.locator("text=users").click({ button: "right" });

    // Verify context menu appears with expected options
    await expect(page.locator("text=Export as JSON")).toBeVisible();
    await expect(page.locator("text=Export as SQL")).toBeVisible();
    await expect(page.locator("text=Truncate")).toBeVisible();
    await expect(page.locator("text=Drop")).toBeVisible();
  });

  test("Right-click row in data table shows row context menu", async ({
    page,
  }) => {
    // Open users table
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    // Right-click on a data row
    const firstRow = page.locator("table tbody tr").first();
    await firstRow.click({ button: "right" });

    // Verify row context menu appears
    await expect(page.locator("text=Delete")).toBeVisible();
    await expect(page.locator("text=Duplicate")).toBeVisible();
    await expect(page.locator("text=Copy as JSON")).toBeVisible();
    await expect(page.locator("text=Copy as SQL INSERT")).toBeVisible();
  });

  test("Right-click saved query shows query context menu", async ({
    page,
  }) => {
    // First, create a saved query
    await page.locator('button[title="New SQL Editor"]').click();
    await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 5000 });

    const editor = page.locator(".cm-content");
    await editor.click();
    await page.keyboard.type("SELECT 1");
    await page.locator('button:has-text("Save")').click();
    await page.waitForTimeout(1000);

    // Find the saved query in sidebar and right-click
    const savedQuery = page.locator("text=Untitled").first();
    await savedQuery.click({ button: "right" });

    // Verify query context menu
    await expect(page.locator("text=Rename")).toBeVisible();
    await expect(page.locator("text=Duplicate")).toBeVisible();
    await expect(page.locator("text=Delete")).toBeVisible();
  });
});
