// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="index.html"><strong aria-hidden="true">1.</strong> Cookbook Overview</a></li><li class="chapter-item expanded "><a href="introduction.html"><strong aria-hidden="true">2.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="testing-patterns/index.html"><strong aria-hidden="true">3.</strong> Testing Patterns</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="testing-patterns/aaa-pattern.html"><strong aria-hidden="true">3.1.</strong> Pattern 1: AAA Pattern</a></li><li class="chapter-item expanded "><a href="testing-patterns/error-path-testing.html"><strong aria-hidden="true">3.2.</strong> Pattern 2: Error Path Testing</a></li><li class="chapter-item expanded "><a href="testing-patterns/boundary-conditions.html"><strong aria-hidden="true">3.3.</strong> Pattern 3: Boundary Conditions</a></li><li class="chapter-item expanded "><a href="testing-patterns/resource-cleanup.html"><strong aria-hidden="true">3.4.</strong> Pattern 4: Resource Cleanup</a></li><li class="chapter-item expanded "><a href="testing-patterns/real-collaborators.html"><strong aria-hidden="true">3.5.</strong> Pattern 5: Real Collaborators</a></li></ol></li><li class="chapter-item expanded "><a href="architecture-patterns/index.html"><strong aria-hidden="true">4.</strong> Architecture Patterns</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="architecture-patterns/generic-base.html"><strong aria-hidden="true">4.1.</strong> Pattern 6: Generic Base Layer</a></li><li class="chapter-item expanded "><a href="architecture-patterns/extension-layer.html"><strong aria-hidden="true">4.2.</strong> Pattern 7: Extension Layer</a></li><li class="chapter-item expanded "><a href="architecture-patterns/composition-over-duplication.html"><strong aria-hidden="true">4.3.</strong> Pattern 8: Composition Over Duplication</a></li><li class="chapter-item expanded "><a href="architecture-patterns/single-source-of-truth.html"><strong aria-hidden="true">4.4.</strong> Pattern 9: Single Source of Truth</a></li><li class="chapter-item expanded "><a href="architecture-patterns/capability-groups.html"><strong aria-hidden="true">4.5.</strong> Pattern 10: Capability Grouping</a></li></ol></li><li class="chapter-item expanded "><a href="design-patterns/index.html"><strong aria-hidden="true">5.</strong> Design Patterns</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="design-patterns/zero-cost-abstractions.html"><strong aria-hidden="true">5.1.</strong> Pattern 11: Zero-Cost Abstractions</a></li><li class="chapter-item expanded "><a href="design-patterns/type-safety-patterns.html"><strong aria-hidden="true">5.2.</strong> Pattern 12: Type Safety with GATs</a></li><li class="chapter-item expanded "><a href="design-patterns/sealed-traits.html"><strong aria-hidden="true">5.3.</strong> Pattern 13: Sealed Traits for API Safety</a></li><li class="chapter-item expanded "><a href="design-patterns/compile-time-validation.html"><strong aria-hidden="true">5.4.</strong> Pattern 14: Compile-Time Validation</a></li><li class="chapter-item expanded "><a href="design-patterns/type-state-pattern.html"><strong aria-hidden="true">5.5.</strong> Pattern 15: Type State Enforcement</a></li></ol></li><li class="chapter-item expanded "><a href="design-patterns/new-patterns.html"><strong aria-hidden="true">6.</strong> New Patterns from Practice</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="design-patterns/fixture-lifecycle.html"><strong aria-hidden="true">6.1.</strong> Pattern 16: Fixture Lifecycle Management</a></li><li class="chapter-item expanded "><a href="design-patterns/builder-test-data.html"><strong aria-hidden="true">6.2.</strong> Pattern 17: Builder-Driven Test Data</a></li><li class="chapter-item expanded "><a href="design-patterns/timeout-defense.html"><strong aria-hidden="true">6.3.</strong> Pattern 18: Timeout Defense in Depth</a></li><li class="chapter-item expanded "><a href="design-patterns/feature-gating.html"><strong aria-hidden="true">6.4.</strong> Pattern 19: Feature Gate Slices</a></li><li class="chapter-item expanded "><a href="design-patterns/macro-enforcement.html"><strong aria-hidden="true">6.5.</strong> Pattern 20: Macro Pattern Enforcement</a></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
