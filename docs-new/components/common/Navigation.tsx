import { Navbar } from "nextra-theme-docs";

const Logo = () => {
  return (
    <svg
      width="98"
      height="29"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className="text-black dark:text-white"
    >
      <path d="M26.133 8.038 14.003 1v4.87l12.13 7.007v-4.84Z" fill="#FF6492" />
      <path
        d="M26.133 12.877 22.494 15 14 10.081l-4.853 2.8-3.64-1.967 8.495-5.044 12.131 7.007Z"
        fill="#141446"
      />
      <path d="m9.147 12.881-3.64-1.967V15l3.64-2.119Z" fill="#A14474" />
      <path
        d="M1.867 21.962 14 15l12.133 6.962L14 29 1.867 21.962Z"
        fill="#141446"
      />
      <path d="M26.133 17.13 14 10.005V15l12.133 6.962V17.13Z" fill="#FF6492" />
      <path
        d="M5.507 15v-4.086l8.495-5.044V1L1.867 8.038v13.924L14 15v-4.995L5.507 15Z"
        fill="#7A77FF"
      />
      <path
        d="M33.66 15.628c0-4.21 3.339-7.485 7.546-7.485 2.7 0 4.789 1.316 6.008 3.332l-2.147 1.696c-.93-1.316-2.119-2.193-3.83-2.193-2.613 0-4.47 2.047-4.47 4.648 0 2.66 1.857 4.676 4.47 4.676 1.683 0 2.873-.846 3.83-2.162l2.148 1.667c-1.248 2.016-3.34 3.332-6.009 3.332-4.207.002-7.546-3.273-7.546-7.511ZM49.856 16.797v-8.33h3.018v8.564c0 2.016 1.421 3.273 3.222 3.273 1.742 0 3.135-1.257 3.135-3.273V8.466h3.048v8.332c0 3.888-2.7 6.343-6.181 6.343-3.543 0-6.242-2.455-6.242-6.343ZM80.77 15.656c0 4.268-2.932 7.485-6.967 7.485-2.09 0-3.919-.847-4.992-2.309v1.988h-2.844V.894h3.048v9.326c1.075-1.315 2.786-2.075 4.76-2.075 4.063-.002 6.994 3.243 6.994 7.511Zm-3.136-.028c0-2.778-1.915-4.677-4.355-4.677-2.147 0-4.296 1.462-4.296 4.707 0 3.274 2.177 4.677 4.296 4.677 2.468 0 4.355-1.93 4.355-4.707ZM96.53 16.651H85.906c.408 2.368 2.206 3.771 4.644 3.771 1.655 0 2.96-.644 4.15-1.695l1.453 2.105c-1.51 1.432-3.426 2.309-5.69 2.309-4.411 0-7.663-3.245-7.663-7.485 0-4.209 3.221-7.513 7.373-7.513 3.802 0 6.588 2.806 6.588 6.577 0 .79-.145 1.552-.231 1.931Zm-10.565-2.398h7.75c-.059-2.252-1.712-3.509-3.657-3.509-2.004.002-3.63 1.375-4.093 3.51Z"
        fill="currentColor"
      />
    </svg>
  );
};

export const Navigation = () => {
  return (
    <div className="w-full p-4 sticky top-0 bg-white dark:bg-[#111111] h-16 flex items-center border-b border-neutral-200 dark:border-neutral-800">
      <Logo />

      <Navbar flatDirectories={[]} items={[]} />
    </div>
  );
};
