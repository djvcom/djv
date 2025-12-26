import { test, expect } from "@playwright/test";

/**
 * US-8: Empty results handling
 *
 * As a visitor, I want to see a helpful message when no projects match
 * so that I know to try different filters.
 */

test.describe("Empty States", () => {
  test("shows empty message when no projects match filters", async ({ page }) => {
    // Navigate with a filter combination unlikely to have results
    await page.goto("/?kind=npm&language=Nix");

    // Should show empty state
    const emptyState = page.locator(".project-grid-empty, .projects-placeholder");
    await expect(emptyState).toBeVisible();
  });

  test("'View all projects' link is visible", async ({ page }) => {
    await page.goto("/");

    const archiveLink = page.locator('a:has-text("View all projects")');
    await expect(archiveLink).toBeVisible();
  });

  test("'View all projects' link navigates correctly", async ({ page }) => {
    await page.goto("/");

    await page.click('a:has-text("View all projects")');

    await expect(page).toHaveURL(/kind=all/);
  });
});
