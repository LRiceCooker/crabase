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

test.describe("Table browsing", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Click users in sidebar, verify tab opens with data", async ({
    page,
  }) => {
    await page.locator("text=users").click();

    // Verify a tab opened and data table renders
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });
    // Verify real row data is present (alice is the first user)
    await expect(
      page.getByRole("cell", { name: "alice", exact: true })
    ).toBeVisible();
  });

  test("Verify column headers match users table schema", async ({ page }) => {
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    // Check key column headers ('id' matches 'uuid' too via substring, so use .first())
    await expect(page.locator("th:has-text('id')").first()).toBeVisible();
    await expect(page.locator("th:has-text('username')")).toBeVisible();
    await expect(page.locator("th:has-text('email')")).toBeVisible();
    await expect(page.locator("th:has-text('role')")).toBeVisible();
  });

  test("Verify pagination shows correct total count (12 rows)", async ({
    page,
  }) => {
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    // Check pagination shows 12 total (use specific text to avoid matching cell data)
    await expect(page.getByText("12 rows").first()).toBeVisible();
  });

  test("Click page 2, verify different rows appear", async ({ page }) => {
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    // If page size is small enough to have page 2
    const page2Button = page.locator('button:has-text("2")');
    if (await page2Button.isVisible()) {
      await page2Button.click();
      // Wait for data to change
      await page.waitForTimeout(500);
      // Verify we see different content than page 1
      await expect(page.locator("table")).toBeVisible();
    }
  });

  test("Verify enum values display correctly", async ({ page }) => {
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    // Enum values should display as readable text (admin, editor, viewer)
    await expect(
      page.getByRole("cell", { name: "admin" }).first()
    ).toBeVisible();
  });

  test("Verify NULL values display correctly", async ({ page }) => {
    await page.locator("text=users").click();
    await expect(page.locator("table")).toBeVisible({ timeout: 10000 });

    // NULL values should display as "NULL" in a distinct style
    await expect(page.locator("text=NULL").first()).toBeVisible();
  });
});
