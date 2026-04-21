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

/** Open a new SQL editor, clear any pre-loaded content, and type SQL */
async function typeInNewEditor(page, sql: string) {
  await page.locator('button[title="New SQL Editor"]').click();
  await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 5000 });

  const editor = page.locator(".cm-content");
  await editor.click();
  // Select all to clear any pre-loaded content (e.g. from a saved query with same name)
  await page.keyboard.press("Meta+A");
  await page.keyboard.type(sql);
}

test.describe("SQL Editor", () => {
  test.beforeEach(async ({ page }) => {
    await connectToDb(page);
  });

  test("Open new SQL editor tab, verify editor visible", async ({ page }) => {
    await page.locator('button[title="New SQL Editor"]').click();
    await expect(page.locator(".cm-editor")).toBeVisible({ timeout: 5000 });
  });

  test("Type and run a SELECT query, verify results", async ({ page }) => {
    await typeInNewEditor(page, "SELECT * FROM users WHERE role = 'admin'");

    // Click Run
    await page.locator('button:has-text("Run")').click();

    // Wait for results table to appear
    await expect(page.locator("table").last()).toBeVisible({ timeout: 10000 });

    // Verify 3 admin rows (alice, diana, heidi)
    await expect(
      page.getByRole("cell", { name: "alice", exact: true })
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: "diana", exact: true })
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: "heidi", exact: true })
    ).toBeVisible();
  });

  test("Run multi-statement query, verify statement selector", async ({
    page,
  }) => {
    await typeInNewEditor(
      page,
      "SELECT * FROM users WHERE role = 'admin'; SELECT * FROM products LIMIT 3"
    );

    // Click Run
    await page.locator('button:has-text("Run")').click();

    // Verify multi-statement navigator appears (shows "#1 SELECT..." and "#2 SELECT..." buttons)
    await expect(page.locator("text=/#1/").first()).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator("text=/#2/").first()).toBeVisible();
  });

  test("Save query, verify it appears in sidebar", async ({ page }) => {
    await typeInNewEditor(page, "SELECT 1");

    // Save via button
    await page.locator('button:has-text("Save")').click();

    // Wait for save to complete
    await page.waitForTimeout(1000);

    // The query should appear in the sidebar saved queries section
    await expect(page.getByText(/Untitled-\d+/).first()).toBeVisible();
  });
});
