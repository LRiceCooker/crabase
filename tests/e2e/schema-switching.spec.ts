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

test.describe("Schema switching", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Switch to test_schema, verify sidebar updates", async ({ page }) => {
    // Find schema select in header and change to test_schema
    const schemaSelect = page.locator("select").first();
    await schemaSelect.selectOption("test_schema");

    // Wait for sidebar to update
    await page.waitForTimeout(2000);

    // Verify test_schema tables appear (tasks table)
    await expect(page.locator("text=tasks")).toBeVisible({ timeout: 10000 });

    // Public tables should not be visible anymore
    await expect(page.locator("text=users")).not.toBeVisible();
  });

  test("Click table in test_schema, verify data loads", async ({ page }) => {
    const schemaSelect = page.locator("select").first();
    await schemaSelect.selectOption("test_schema");

    await expect(page.locator("text=tasks")).toBeVisible({ timeout: 10000 });

    // Click tasks table
    await page.locator("text=tasks").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    // Verify task data is loaded (check for columns or data)
    await expect(page.locator("th:has-text('title')")).toBeVisible();
  });

  test("Switch back to public, verify public tables return", async ({
    page,
  }) => {
    // Switch to test_schema
    const schemaSelect = page.locator("select").first();
    await schemaSelect.selectOption("test_schema");
    await expect(page.locator("text=tasks")).toBeVisible({ timeout: 10000 });

    // Switch back to public
    await schemaSelect.selectOption("public");
    await page.waitForTimeout(2000);

    // Verify public tables are back
    await expect(page.locator("text=users")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("text=products")).toBeVisible();
    await expect(page.locator("text=events")).toBeVisible();
  });
});
