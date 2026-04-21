import { test, expect } from "./fixtures";

const CONNECTION_STRING = "postgresql://test:test@localhost:5433/crabase_test";

async function connectAndOpenUsers(page) {
  await page.goto("/");
  const input = page.locator(
    'input[placeholder="postgresql://user:password@host:port/dbname"]'
  );
  await input.fill(CONNECTION_STRING);
  await page.locator('button:has-text("Next")').click();
  await expect(page.locator("text=Connection details")).toBeVisible();
  await page.locator('button:has-text("Connect")').click();
  await expect(page.locator("text=users")).toBeVisible({ timeout: 15000 });
  await page.locator("text=users").click();
  await expect(page.locator("table")).toBeVisible({ timeout: 10000 });
}

test.describe("Filters and sorting", () => {
  test.beforeEach(async ({ page }) => {
    await connectAndOpenUsers(page);
  });

  test("Add filter for role=admin, verify filtered results", async ({
    page,
  }) => {
    // Click add filter button
    await page.locator('button[title="Add filter"]').click();

    // The filter chip appears with column select, operator select, and value input
    const valueInput = page.locator('input[placeholder="value"]');
    await expect(valueInput).toBeVisible({ timeout: 3000 });
    const chipSpan = valueInput.locator("..");
    const columnSelect = chipSpan.locator("select").first();

    // Select column "role" (triggers an intermediate refetch with empty value that may error)
    await columnSelect.selectOption("role");
    // Wait for the intermediate fetch to settle before committing the value
    await page.waitForTimeout(1500);
    // Fill value and press Enter → triggers refetch with complete filter
    await valueInput.fill("admin");
    await valueInput.press("Enter");

    // Wait for filtered results
    await expect(
      page.getByRole("cell", { name: "alice", exact: true })
    ).toBeVisible({ timeout: 10000 });
    await expect(
      page.getByRole("cell", { name: "diana", exact: true })
    ).toBeVisible();
    await expect(
      page.getByRole("cell", { name: "heidi", exact: true })
    ).toBeVisible();
  });

  test("Click column header to sort ascending", async ({ page }) => {
    // Click on "username" header to sort
    const header = page.locator("th:has-text('username')");
    await header.click();

    // Wait for sort to apply
    await page.waitForTimeout(500);

    // First row should be alice (alphabetically first)
    const firstRow = page.locator("table tbody tr").first();
    await expect(firstRow).toContainText("alice");
  });

  test("Click column header again for descending sort", async ({ page }) => {
    // Click username header twice for descending
    const header = page.locator("th:has-text('username')");
    await header.click();
    await page.waitForTimeout(300);
    await header.click();
    await page.waitForTimeout(500);

    // First row should be lara (alphabetically last)
    const firstRow = page.locator("table tbody tr").first();
    await expect(firstRow).toContainText("lara");
  });
});
