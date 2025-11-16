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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="index.html"><strong aria-hidden="true">1.</strong> Application Guide</a></li><li class="chapter-item expanded "><a href="introduction.html"><strong aria-hidden="true">2.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="choosing-your-path.html"><strong aria-hidden="true">3.</strong> Choosing Your Learning Path</a></li><li class="chapter-item expanded affix "><li class="part-title">Tutorials</li><li class="chapter-item expanded "><a href="tutorials/getting-started.html"><strong aria-hidden="true">4.</strong> Quick Start (25 minutes)</a></li><li class="chapter-item expanded "><a href="tutorials/fixtures-tutorial.html"><strong aria-hidden="true">5.</strong> Fixtures Deep Dive (15 minutes)</a></li><li class="chapter-item expanded "><a href="tutorials/cli-app-tutorial.html"><strong aria-hidden="true">6.</strong> CLI Application (45 minutes)</a></li><li class="chapter-item expanded "><a href="tutorials/web-service-tutorial.html"><strong aria-hidden="true">7.</strong> REST Web Service (50 minutes)</a></li><li class="chapter-item expanded affix "><li class="part-title">API Reference</li><li class="chapter-item expanded "><a href="reference/fixtures-api.html"><strong aria-hidden="true">8.</strong> TestFixture API</a></li><li class="chapter-item expanded affix "><li class="part-title">Core Testing Patterns</li><li class="chapter-item expanded "><a href="core/index.html"><strong aria-hidden="true">9.</strong> Core Testing Patterns</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="core/fixtures.html"><strong aria-hidden="true">9.1.</strong> Getting Started with Fixtures</a></li><li class="chapter-item expanded "><a href="core/data-builders.html"><strong aria-hidden="true">9.2.</strong> Building Test Data</a></li><li class="chapter-item expanded "><a href="core/assertions.html"><strong aria-hidden="true">9.3.</strong> Assertions &amp; Verification</a></li><li class="chapter-item expanded "><a href="core/error-paths.html"><strong aria-hidden="true">9.4.</strong> Error Path Testing</a></li></ol></li><li class="chapter-item expanded "><a href="advanced/index.html"><strong aria-hidden="true">10.</strong> Advanced Testing Techniques</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="advanced/property-testing.html"><strong aria-hidden="true">10.1.</strong> Property-Based Testing</a></li><li class="chapter-item expanded "><a href="advanced/mutation-testing.html"><strong aria-hidden="true">10.2.</strong> Mutation Testing</a></li><li class="chapter-item expanded "><a href="advanced/snapshot-testing.html"><strong aria-hidden="true">10.3.</strong> Snapshot Testing</a></li><li class="chapter-item expanded "><a href="advanced/cli-testing.html"><strong aria-hidden="true">10.4.</strong> CLI Testing</a></li><li class="chapter-item expanded "><a href="advanced/concurrency-testing.html"><strong aria-hidden="true">10.5.</strong> Concurrency Testing</a></li></ol></li><li class="chapter-item expanded "><a href="guides/extra-mile.html"><strong aria-hidden="true">11.</strong> The &quot;Go the Extra Mile&quot; Pattern</a></li><li class="chapter-item expanded "><a href="guides/observability.html"><strong aria-hidden="true">12.</strong> Observability &amp; Quality</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="guides/otel.html"><strong aria-hidden="true">12.1.</strong> OTEL Instrumentation</a></li><li class="chapter-item expanded "><a href="guides/weaver.html"><strong aria-hidden="true">12.2.</strong> Weaver Live-Check Validation</a></li><li class="chapter-item expanded "><a href="guides/coverage-performance.html"><strong aria-hidden="true">12.3.</strong> Coverage &amp; Performance</a></li></ol></li><li class="chapter-item expanded "><a href="guides/real-world.html"><strong aria-hidden="true">13.</strong> Real-World Applications</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="guides/cli-application.html"><strong aria-hidden="true">13.1.</strong> Building a CLI Tool</a></li><li class="chapter-item expanded "><a href="guides/web-service.html"><strong aria-hidden="true">13.2.</strong> Testing a Web Service</a></li><li class="chapter-item expanded "><a href="guides/integration-docker.html"><strong aria-hidden="true">13.3.</strong> Integration Testing with Docker</a></li></ol></li><li class="chapter-item expanded "><a href="guides/best-practices.html"><strong aria-hidden="true">14.</strong> Best Practices &amp; Migration</a></li></ol>';
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
