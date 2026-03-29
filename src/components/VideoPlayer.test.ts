// Unit tests for fileToAssetUrl logic
// Run: npx vitest run src/components/VideoPlayer.test.ts

function fileToAssetUrl(filePath: string, isWindows: boolean): string {
  const normalized = filePath.replace(/\\/g, "/");
  const segments = normalized.split("/");
  const encoded = segments
    .map((segment) => (segment === "" ? "" : encodeURIComponent(segment)))
    .join("/");
  const path = encoded.startsWith("/") ? encoded : `/${encoded}`;
  if (isWindows) {
    return `https://asset.localhost${path}`;
  }
  return `asset://localhost${path}`;
}

import { describe, it, expect } from "vitest";

describe("fileToAssetUrl", () => {
  // macOS paths
  it("macOS absolute path", () => {
    expect(fileToAssetUrl("/Volumes/data/video.mp4", false)).toBe(
      "asset://localhost/Volumes/data/video.mp4"
    );
  });

  it("macOS path with special chars (@)", () => {
    expect(
      fileToAssetUrl("/Volumes/data/hhd800.com@FC2-PPV-123.mp4", false)
    ).toBe(
      "asset://localhost/Volumes/data/hhd800.com%40FC2-PPV-123.mp4"
    );
  });

  it("macOS path with spaces", () => {
    expect(fileToAssetUrl("/Users/me/My Videos/test.mp4", false)).toBe(
      "asset://localhost/Users/me/My%20Videos/test.mp4"
    );
  });

  it("macOS path with #", () => {
    expect(fileToAssetUrl("/data/video#1.mp4", false)).toBe(
      "asset://localhost/data/video%231.mp4"
    );
  });

  // Windows local paths
  it("Windows drive path with backslashes", () => {
    expect(fileToAssetUrl("C:\\Users\\me\\video.mp4", true)).toBe(
      "https://asset.localhost/C%3A/Users/me/video.mp4"
    );
  });

  it("Windows drive path with forward slashes", () => {
    expect(fileToAssetUrl("C:/Users/me/video.mp4", true)).toBe(
      "https://asset.localhost/C%3A/Users/me/video.mp4"
    );
  });

  // Windows UNC paths
  it("Windows UNC path", () => {
    expect(
      fileToAssetUrl("\\\\SERVER\\share\\folder\\video.mp4", true)
    ).toBe(
      "https://asset.localhost//SERVER/share/folder/video.mp4"
    );
  });

  it("Windows UNC path with special chars", () => {
    expect(
      fileToAssetUrl(
        "\\\\UGNAS01\\personal_folder\\v\\hosi@test.mp4",
        true
      )
    ).toBe(
      "https://asset.localhost//UGNAS01/personal_folder/v/hosi%40test.mp4"
    );
  });

  // Linux paths
  it("Linux absolute path", () => {
    expect(fileToAssetUrl("/home/user/videos/test.mp4", false)).toBe(
      "asset://localhost/home/user/videos/test.mp4"
    );
  });

  it("Linux path with Japanese chars", () => {
    expect(fileToAssetUrl("/home/user/動画/テスト.mp4", false)).toBe(
      "asset://localhost/home/user/%E5%8B%95%E7%94%BB/%E3%83%86%E3%82%B9%E3%83%88.mp4"
    );
  });
});
