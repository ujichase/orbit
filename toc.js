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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded "><a href="starting/starting.html"><strong aria-hidden="true">1.</strong> Getting Started</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="starting/motivation.html"><strong aria-hidden="true">1.1.</strong> Motivation</a></li><li class="chapter-item expanded "><a href="starting/installing.html"><strong aria-hidden="true">1.2.</strong> Installing</a></li><li class="chapter-item expanded "><a href="starting/upgrading.html"><strong aria-hidden="true">1.3.</strong> Upgrading</a></li></ol></li><li class="chapter-item expanded "><a href="tutorials/tutorials.html"><strong aria-hidden="true">2.</strong> Tutorials</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="tutorials/first_project.html"><strong aria-hidden="true">2.1.</strong> First Project: Gates</a></li><li class="chapter-item expanded "><a href="tutorials/dependencies.html"><strong aria-hidden="true">2.2.</strong> Dependencies: Half adder</a></li><li class="chapter-item expanded "><a href="tutorials/gates_revisited.html"><strong aria-hidden="true">2.3.</strong> Gates: Revisited</a></li><li class="chapter-item expanded "><a href="tutorials/final_project.html"><strong aria-hidden="true">2.4.</strong> Final Project: Full adder</a></li></ol></li><li class="chapter-item expanded "><a href="user/user.html"><strong aria-hidden="true">3.</strong> User Guide</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="user/overview.html"><strong aria-hidden="true">3.1.</strong> Overview</a></li></ol></li><li class="chapter-item expanded "><a href="topic/topic.html"><strong aria-hidden="true">4.</strong> Topic Guide</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="topic/overview.html"><strong aria-hidden="true">4.1.</strong> Overview</a></li><li class="chapter-item expanded "><a href="topic/package_management.html"><strong aria-hidden="true">4.2.</strong> Agile Package Management</a></li><li class="chapter-item expanded "><a href="topic/extensible_builds.html"><strong aria-hidden="true">4.3.</strong> Extensible Builds</a></li><li class="chapter-item expanded "><a href="topic/catalog.html"><strong aria-hidden="true">4.4.</strong> Catalog</a></li><li class="chapter-item expanded "><a href="topic/ip.html"><strong aria-hidden="true">4.5.</strong> Ip</a></li><li class="chapter-item expanded "><a href="topic/targets.html"><strong aria-hidden="true">4.6.</strong> Targets</a></li><li class="chapter-item expanded "><a href="topic/protocols.html"><strong aria-hidden="true">4.7.</strong> Protocols</a></li><li class="chapter-item expanded "><a href="topic/channels.html"><strong aria-hidden="true">4.8.</strong> Channels</a></li><li class="chapter-item expanded "><a href="topic/orbitlock.html"><strong aria-hidden="true">4.9.</strong> Orbit.lock</a></li><li class="chapter-item expanded "><a href="topic/visibility.html"><strong aria-hidden="true">4.10.</strong> File Visibility</a></li><li class="chapter-item expanded "><a href="topic/swapping.html"><strong aria-hidden="true">4.11.</strong> String Swapping</a></li><li class="chapter-item expanded "><a href="topic/dst.html"><strong aria-hidden="true">4.12.</strong> Dynamic Symbol Transformation</a></li></ol></li><li class="chapter-item expanded "><a href="reference/reference.html"><strong aria-hidden="true">5.</strong> Reference</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="reference/manifest.html"><strong aria-hidden="true">5.1.</strong> Manifest</a></li><li class="chapter-item expanded "><a href="reference/names.html"><strong aria-hidden="true">5.2.</strong> Names</a></li><li class="chapter-item expanded "><a href="reference/versions.html"><strong aria-hidden="true">5.3.</strong> Versions</a></li><li class="chapter-item expanded "><a href="reference/filesets.html"><strong aria-hidden="true">5.4.</strong> Filesets</a></li><li class="chapter-item expanded "><a href="reference/blueprint.html"><strong aria-hidden="true">5.5.</strong> Blueprint</a></li><li class="chapter-item expanded "><a href="reference/environment_variables.html"><strong aria-hidden="true">5.6.</strong> Environment Variables</a></li><li class="chapter-item expanded "><a href="reference/configuration.html"><strong aria-hidden="true">5.7.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="reference/json.html"><strong aria-hidden="true">5.8.</strong> JSON Output</a></li><li class="chapter-item expanded "><a href="reference/command_line.html"><strong aria-hidden="true">5.9.</strong> Command Line</a></li></ol></li><li class="chapter-item expanded "><a href="commands/commands.html"><strong aria-hidden="true">6.</strong> Commands</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="commands/new.html"><strong aria-hidden="true">6.1.</strong> orbit new</a></li><li class="chapter-item expanded "><a href="commands/init.html"><strong aria-hidden="true">6.2.</strong> orbit init</a></li><li class="chapter-item expanded "><a href="commands/info.html"><strong aria-hidden="true">6.3.</strong> orbit info</a></li><li class="chapter-item expanded "><a href="commands/read.html"><strong aria-hidden="true">6.4.</strong> orbit read</a></li><li class="chapter-item expanded "><a href="commands/get.html"><strong aria-hidden="true">6.5.</strong> orbit get</a></li><li class="chapter-item expanded "><a href="commands/tree.html"><strong aria-hidden="true">6.6.</strong> orbit tree</a></li><li class="chapter-item expanded "><a href="commands/lock.html"><strong aria-hidden="true">6.7.</strong> orbit lock</a></li><li class="chapter-item expanded "><a href="commands/test.html"><strong aria-hidden="true">6.8.</strong> orbit test</a></li><li class="chapter-item expanded "><a href="commands/build.html"><strong aria-hidden="true">6.9.</strong> orbit build</a></li><li class="chapter-item expanded "><a href="commands/publish.html"><strong aria-hidden="true">6.10.</strong> orbit publish</a></li><li class="chapter-item expanded "><a href="commands/search.html"><strong aria-hidden="true">6.11.</strong> orbit search</a></li><li class="chapter-item expanded "><a href="commands/install.html"><strong aria-hidden="true">6.12.</strong> orbit install</a></li><li class="chapter-item expanded "><a href="commands/remove.html"><strong aria-hidden="true">6.13.</strong> orbit remove</a></li><li class="chapter-item expanded "><a href="commands/env.html"><strong aria-hidden="true">6.14.</strong> orbit env</a></li><li class="chapter-item expanded "><a href="commands/config.html"><strong aria-hidden="true">6.15.</strong> orbit config</a></li></ol></li><li class="chapter-item expanded "><a href="glossary.html"><strong aria-hidden="true">7.</strong> Appendix: Glossary</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0];
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
