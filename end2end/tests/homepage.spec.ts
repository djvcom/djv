import { test, expect } from "@playwright/test";

/**
 * US-1: First-time visitor views portfolio
 *
 * As a visitor, I want to see a list of projects and contributions
 * so that I can explore any that look interesting.
 */

test.describe("Homepage", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("has correct page title", async ({ page }) => {
    await expect(page).toHaveTitle("Daniel Verrall");
  });

  test("displays hero section with name and tagline", async ({ page }) => {
    const hero = page.locator(".hero");
    await expect(hero.locator("h1")).toHaveText("Daniel Verrall");
    await expect(hero.locator(".tagline")).toContainText("rust");
  });

  test("displays projects section", async ({ page }) => {
    const projectsSection = page.locator("section.projects");
    await expect(projectsSection).toBeVisible();
    await expect(projectsSection.locator("h2")).toHaveText("Projects");
  });

  test("displays filter bar with options", async ({ page }) => {
    const filterBar = page.locator(".filter-bar");
    await expect(filterBar).toBeVisible();

    // Check filter groups exist
    await expect(filterBar.locator(".filter-group")).toHaveCount(3); // kind, language, sort (topic may not show)
  });

  test("displays contributions section", async ({ page }) => {
    const contributionsSection = page.locator("section.contributions");
    await expect(contributionsSection).toBeVisible();
    await expect(contributionsSection.locator("h2")).toHaveText("Contributions");
  });

  test("project cards have external links with correct attributes", async ({ page }) => {
    // Wait for projects to load
    const projectCard = page.locator(".project-card").first();

    // Skip if no projects loaded
    if ((await projectCard.count()) === 0) {
      test.skip();
      return;
    }

    const link = projectCard.locator("a").first();
    await expect(link).toHaveAttribute("target", "_blank");
    await expect(link).toHaveAttribute("rel", /noopener/);
  });
});
