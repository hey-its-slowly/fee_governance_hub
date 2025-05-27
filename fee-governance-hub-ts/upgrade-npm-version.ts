import * as fs from "fs";
import * as path from "path";

function incrementPatchVersion() {
  try {
    const packageJsonPath = path.resolve(__dirname, "package.json");
    console.log(`Reading package.json from: ${packageJsonPath}`);

    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));

    // Split the version string into parts
    const versionParts = packageJson.version.split(".").map(Number);

    // Increment the last part of the version
    versionParts[2] += 1;

    // Join the parts back into a version string
    packageJson.version = versionParts.join(".");

    fs.writeFileSync(
      packageJsonPath,
      JSON.stringify(packageJson, null, 2) + "\n"
    );

    console.log(`Updated version to: ${packageJson.version}`);
  } catch (error) {
    console.error("An error occurred:", error);
  }
}

// Example usage
incrementPatchVersion();
