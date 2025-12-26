import { test, expect } from "@playwright/test";

/**
 * US-2, US-3, US-4: Filtering projects
 *
 * As a visitor, I want to filter projects by type, language, or topic
 * so that I can find specific kinds of work.
 */

test.describe("Project Filtering", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test.describe("Kind filter", () => {
    test("clicking 'crates' filters to crates only", async ({ page }) => {
      await page.click('button:has-text("crates")');

      await expect(page).toHaveURL(/kind=crate/);
      await expect(page.locator('button:has-text("crates")')).toHaveClass(/active/);
    });

    test("clicking 'repos' filters to repos only", async ({ page }) => {
      await page.click('button:has-text("repos")');

      await expect(page).toHaveURL(/kind=repo/);
      await expect(page.locator('button:has-text("repos")')).toHaveClass(/active/);
    });

    test("clicking 'all' resets kind filter", async ({ page }) => {
      // First apply a filter
      await page.click('button:has-text("crates")');
      await expect(page).toHaveURL(/kind=crate/);

      // Then reset
      await page.click('button:has-text("all")');
      await expect(page).not.toHaveURL(/kind=/);
    });
  });

  test.describe("Language filter", () => {
    test("clicking 'rust' filters by Rust language", async ({ page }) => {
      await page.click('button:has-text("rust")');

      await expect(page).toHaveURL(/language=Rust/);
      await expect(page.locator('button:has-text("rust")')).toHaveClass(/active/);
    });

    test("clicking 'any' resets language filter", async ({ page }) => {
      await page.click('button:has-text("rust")');
      await expect(page).toHaveURL(/language=/);

      await page.click('button:has-text("any")');
      await expect(page).not.toHaveURL(/language=/);
    });
  });

  test.describe("Combined filters", () => {
    test("multiple filters can be active simultaneously", async ({ page }) => {
      await page.click('button:has-text("crates")');
      await page.click('button:has-text("rust")');

      await expect(page).toHaveURL(/kind=crate/);
      await expect(page).toHaveURL(/language=Rust/);
    });

    test("changing one filter preserves others", async ({ page }) => {
      // Apply both filters
      await page.click('button:has-text("crates")');
      await page.click('button:has-text("rust")');

      // Change kind filter
      await page.click('button:has-text("repos")');

      // Language should still be set
      await expect(page).toHaveURL(/kind=repo/);
      await expect(page).toHaveURL(/language=Rust/);
    });
  });
});
