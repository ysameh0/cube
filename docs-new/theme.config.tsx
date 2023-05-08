import type { DocsThemeConfig } from "nextra-theme-docs";
import { components } from "@/components/mdx";
import { CubeLogo } from "@/components/common/CubeLogo";
import { Footer } from "@/components/common/Footer";
import { SearchTrigger } from "@cube-dev/marketing-ui";

const repo = "https://github.com/cubedevinc/cube-docs-new";
const branch = "main";
const path = "";

const config: DocsThemeConfig = {
  logo: CubeLogo,
  docsRepositoryBase: `${repo}/blob/${branch}/${path}`,
  project: {
    link: repo,
  },
  useNextSeoProps() {
    return {
      titleTemplate: "%s | Cube Docs",
    };
  },
  primaryHue: {
    light: 251,
    dark: 342,
  },
  components,
  sidebar: {
    defaultMenuCollapseLevel: 1,
  },
  search: {
    component: <SearchTrigger>Search</SearchTrigger>,
  },
  footer: {
    component: <Footer />,
  },
  darkMode: false
};

export default config;
