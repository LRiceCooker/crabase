import { test, expect } from "./fixtures";

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

// Helper: find a tab in the tab bar by its title text
function tabByName(page, name: string) {
  // Tab bar is identified by overflow-x-auto (unique to tab bar)
  return page
    .locator(".overflow-x-auto.h-10 div:has-text('" + name + "')")
    .first();
}

test.describe("Tabs", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Open multiple table tabs, switch between them", async ({ page }) => {
    // Open users tab
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });
    await expect(
      page.getByRole("cell", { name: "alice", exact: true })
    ).toBeVisible();

    // Open products tab
    await page.locator("text=products").click();
    await expect(page.locator("th:has-text('name')")).toBeVisible({
      timeout: 10000,
    });

    // Switch back to users tab (click tab header in tab bar)
    await tabByName(page, "users").click();
    await expect(
      page.getByRole("cell", { name: "alice", exact: true })
    ).toBeVisible();
  });

  test("Close a tab, verify adjacent tab becomes active", async ({ page }) => {
    // Open two table tabs
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    await page.locator("text=products").click();
    await expect(page.locator("th:has-text('price')")).toBeVisible({
      timeout: 10000,
    });

    // Close the products tab — find the close button (the button inside the tab div)
    const productsTab = tabByName(page, "products");
    await productsTab.locator("button").click();

    // Users tab should now be active
    await expect(
      page.getByRole("cell", { name: "alice", exact: true })
    ).toBeVisible({ timeout: 5000 });
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
    await tabByName(page, "users").click();
    await expect(
      page.getByRole("cell", { name: "alice", exact: true })
    ).toBeVisible();

    // Switch to SQL tab — click the icon (not the title, which starts rename mode)
    await tabByName(page, "Untitled").locator("svg").first().click();
    await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 10000 });
  });
});
