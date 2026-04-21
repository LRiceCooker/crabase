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

test.describe("Inline editing", () => {
  test.beforeEach(async ({ page }) => {
    await connectAndOpenUsers(page);
  });

  test("Click cell to enter edit mode", async ({ page }) => {
    // Click on a bio cell (text field, editable)
    const bioCell = page.locator("td:has-text('Software engineer')");
    await bioCell.click();

    // Verify an input/textarea appears (edit mode)
    await expect(
      page.locator("td input, td textarea").first()
    ).toBeVisible();
  });

  test("Edit a cell, verify dirty bar appears", async ({ page }) => {
    // Click on a bio cell
    const bioCell = page.locator("td:has-text('Software engineer')");
    await bioCell.click();

    // Type a new value
    const editor = page.locator("td input, td textarea").first();
    await editor.fill("Senior engineer");
    await editor.press("Enter");

    // Verify dirty bar appears
    await expect(page.locator("text=Save changes")).toBeVisible();
    await expect(page.locator("text=Discard")).toBeVisible();
  });

  test("Edit and discard restores original value", async ({ page }) => {
    const bioCell = page.locator("td:has-text('Software engineer')");
    await bioCell.click();

    const editor = page.locator("td input, td textarea").first();
    await editor.fill("Changed value");
    await editor.press("Enter");

    // Verify dirty bar
    await expect(page.locator("text=Save changes")).toBeVisible();

    // Click Discard
    await page.locator("text=Discard").click();

    // Verify original value is back and dirty bar gone
    await expect(page.locator("td:has-text('Software engineer')")).toBeVisible();
    await expect(page.locator("text=Save changes")).not.toBeVisible();
  });

  test("Edit and save persists to DB", async ({ page }) => {
    const bioCell = page.locator("td:has-text('Content writer')");
    await bioCell.click();

    const editor = page.locator("td input, td textarea").first();
    await editor.fill("Senior content writer");
    await editor.press("Enter");

    // Save
    await page.locator("text=Save changes").click();

    // Wait for save + refetch: table reloads (loading spinner then table reappears)
    await expect(page.locator("table")).toBeVisible({ timeout: 15000 });

    // Verify the new value is displayed
    await expect(
      page.locator("td:has-text('Senior content writer')")
    ).toBeVisible({ timeout: 5000 });
  });

  test("Saved value persists after page reload", async ({ page }) => {
    // Edit and save
    const bioCell = page.locator("td:has-text('UI designer')");
    await bioCell.click();

    const editor = page.locator("td input, td textarea").first();
    await editor.fill("Lead designer");
    await editor.press("Enter");

    await page.locator("text=Save changes").click();

    // Wait for save + refetch: dirty bar disappears, then table reappears with new value
    await expect(page.locator("text=Save changes")).not.toBeVisible({
      timeout: 15000,
    });
    // Wait for save + refetch to complete: new value appears in table
    await expect(
      page.locator("td:has-text('Lead designer')")
    ).toBeVisible({ timeout: 15000 });

    // Reload — app restarts at connection screen, so reconnect
    await page.reload();
    await connectAndOpenUsers(page);

    // Verify the saved value persists in the DB
    await expect(
      page.locator("td:has-text('Lead designer')")
    ).toBeVisible({ timeout: 5000 });
  });
});
