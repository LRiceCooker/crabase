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

test.describe("Tabs", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Open multiple table tabs, switch between them", async ({ page }) => {
    // Open users tab
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("td:has-text('alice')")).toBeVisible();

    // Open products tab
    await page.locator("text=products").click();
    await expect(page.locator("th:has-text('name')")).toBeVisible({
      timeout: 10000,
    });

    // Switch back to users tab (click tab header)
    await page.locator('[role="tab"]:has-text("users")').click();
    await expect(page.locator("td:has-text('alice')")).toBeVisible();
  });

  test("Close a tab, verify adjacent tab becomes active", async ({ page }) => {
    // Open two table tabs
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    await page.locator("text=products").click();
    await expect(page.locator("th:has-text('price')")).toBeVisible({
      timeout: 10000,
    });

    // Close the products tab (find the close button on that tab)
    const productsTab = page.locator('[role="tab"]:has-text("products")');
    const closeBtn = productsTab.locator("button, svg").last();
    await closeBtn.click();

    // Users tab should now be active
    await expect(page.locator("td:has-text('alice')")).toBeVisible();
  });

  test("Open SQL editor tab and table tab, switch correctly", async ({
    page,
  }) => {
    // Open users table
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    // Open SQL editor
    await page.locator('button[title="New SQL Editor"]').click();
    await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 5000 });

    // Switch back to users tab
    await page.locator('[role="tab"]:has-text("users")').click();
    await expect(page.locator("td:has-text('alice')")).toBeVisible();

    // Switch to SQL tab
    await page.locator('[role="tab"]:has-text("Untitled")').click();
    await expect(page.locator(".cm-editor")).toBeVisible();
  });
});
