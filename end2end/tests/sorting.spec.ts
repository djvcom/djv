import { test, expect } from "@playwright/test";

/**
 * US-5: Sorting projects
 *
 * As a visitor, I want to sort projects by popularity, name, or recency
 * so that I can find what I'm looking for more easily.
 */

test.describe("Project Sorting", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("default sort is popularity", async ({ page }) => {
    // Popular button should be active by default
    await expect(page.locator('button:has-text("popular")')).toHaveClass(/active/);
  });

  test("clicking 'name' sorts alphabetically", async ({ page }) => {
    await page.click('button:has-text("name")');

    await expect(page).toHaveURL(/sort=name/);
    await expect(page.locator('button:has-text("name")')).toHaveClass(/active/);
  });

  test("clicking 'recent' sorts by update date", async ({ page }) => {
    await page.click('button:has-text("recent")');

    await expect(page).toHaveURL(/sort=updated/);
    await expect(page.locator('button:has-text("recent")')).toHaveClass(/active/);
  });

  test("sort persists with other filters", async ({ page }) => {
    // Apply sort
    await page.click('button:has-text("name")');

    // Apply kind filter
    await page.click('button:has-text("crates")');

    // Both should be in URL
    await expect(page).toHaveURL(/sort=name/);
    await expect(page).toHaveURL(/kind=crate/);
  });
});
