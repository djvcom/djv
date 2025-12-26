import { test, expect } from "@playwright/test";

/**
 * US-6, US-7: Browser navigation and URL sharing
 *
 * As a visitor, I want browser back/forward to work with filters
 * and I want to share URLs that preserve my filter selection.
 */

test.describe("Navigation", () => {
  test.describe("Browser history", () => {
    test("back button restores previous filter state", async ({ page }) => {
      await page.goto("/");

      // Apply filter
      await page.click('button:has-text("crates")');
      await expect(page).toHaveURL(/kind=crate/);

      // Apply another filter
      await page.click('button:has-text("rust")');
      await expect(page).toHaveURL(/language=Rust/);

      // Go back
      await page.goBack();
      await expect(page).toHaveURL(/kind=crate/);
      await expect(page).not.toHaveURL(/language=/);
    });

    test("forward button works after going back", async ({ page }) => {
      await page.goto("/");

      await page.click('button:has-text("crates")');
      await page.click('button:has-text("rust")');

      await page.goBack();
      await page.goForward();

      await expect(page).toHaveURL(/kind=crate/);
      await expect(page).toHaveURL(/language=Rust/);
    });
  });

  test.describe("Direct URL access", () => {
    test("URL with kind param applies filter on load", async ({ page }) => {
      await page.goto("/?kind=crate");

      await expect(page.locator('button:has-text("crates")')).toHaveClass(/active/);
    });

    test("URL with language param applies filter on load", async ({ page }) => {
      await page.goto("/?language=Rust");

      await expect(page.locator('button:has-text("rust")')).toHaveClass(/active/);
    });

    test("URL with multiple params applies all filters", async ({ page }) => {
      await page.goto("/?kind=crate&language=Rust&sort=name");

      await expect(page.locator('button:has-text("crates")')).toHaveClass(/active/);
      await expect(page.locator('button:has-text("rust")')).toHaveClass(/active/);
      await expect(page.locator('button:has-text("name")')).toHaveClass(/active/);
    });

    test("refresh preserves filter state", async ({ page }) => {
      await page.goto("/");
      await page.click('button:has-text("crates")');
      await page.click('button:has-text("rust")');

      await page.reload();

      await expect(page.locator('button:has-text("crates")')).toHaveClass(/active/);
      await expect(page.locator('button:has-text("rust")')).toHaveClass(/active/);
    });
  });
});
