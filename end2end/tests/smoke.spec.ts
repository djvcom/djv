import { test, expect } from "@playwright/test";

/**
 * Smoke tests for djv.sh
 *
 * Fast, essential tests that verify the site works.
 * Run in <90 seconds. For comprehensive testing, use synthetic
 * monitoring against production.
 */

test.describe("Smoke Tests", () => {
  test("page loads with header and projects", async ({ page }) => {
    await page.goto("/");

    // SSR renders header
    await expect(page.locator(".header")).toBeVisible();
    await expect(page.locator(".header__name")).toBeVisible();

    // Projects section exists (list, placeholder, or empty state)
    await expect(
      page.locator(".project-list, .projects-placeholder, .project-empty")
    ).toBeVisible();
  });

  test("page hydrates and becomes interactive", async ({ page }) => {
    await page.goto("/");

    // Wait for WASM to load
    await page.waitForLoadState("networkidle");

    // Filter toggle should be clickable after hydration
    const filterToggle = page.locator(".filter-toggle");
    await expect(filterToggle).toBeVisible();
    await filterToggle.click();

    // Filter groups should expand
    await expect(page.locator(".filter-groups")).toHaveClass(
      /filter-groups--expanded/
    );
  });

  test("theme toggle switches theme", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");

    const toggle = page.locator(".theme-toggle");
    const html = page.locator("html");

    // Get initial state
    const wasDark = await html.evaluate((el) => el.classList.contains("dark"));

    // Click toggle
    await toggle.click();

    // Theme should change
    if (wasDark) {
      await expect(html).not.toHaveClass(/dark/);
    } else {
      await expect(html).toHaveClass(/dark/);
    }
  });

  test("direct URL with filter params works", async ({ page }) => {
    // SSR should respect query params
    await page.goto("/?kind=crate&sort=name");

    await page.waitForLoadState("networkidle");

    // Expand filters to verify state
    await page.locator(".filter-toggle").click();
    await expect(page.locator(".filter-groups")).toHaveClass(
      /filter-groups--expanded/
    );

    // Correct buttons should be active
    await expect(page.locator('button:has-text("crates")')).toHaveClass(
      /active/
    );
    await expect(page.locator('button:has-text("name")')).toHaveClass(/active/);
  });

  test("navigation to /projects works", async ({ page }) => {
    await page.goto("/");

    const projectsLink = page.locator('.header__nav a[href="/projects"]');
    await expect(projectsLink).toBeVisible();

    await projectsLink.click();
    await expect(page).toHaveURL("/projects");
  });
});
