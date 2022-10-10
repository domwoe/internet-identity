
import { html } from "lit-html";
import { icBadge } from "./icons";

export const footer = html`<footer class="l-footer">
<ul class="l-footer__elements">
    <li class="l-footer__element">
  <a
    aria-label="Internet Computer homepage"
    href="#"
    rel="noopener noreferrer"
    target="_blank"
    >Home</a>
    </li>
    <li class="l-footer__element">
  <a
    aria-label="Internet Computer homepage"
    href="#"
    rel="noopener noreferrer"
    target="_blank"
    >Help</a>
    </li>
    <li class="l-footer__element">
  <a
    class="page-signature"
    aria-label="Internet Computer homepage"
    href="https://internetcomputer.org/"
    rel="noopener noreferrer"
    target="_blank"
    >${icBadge}</a>
    </li>
    </ul>
</footer>`;
