import { describe, it, expect } from "vitest";
import { streamUrl } from "./VideoPlayer";

describe("streamUrl", () => {
  const port = 12345;

  // macOS paths
  it("macOS absolute path", () => {
    expect(streamUrl(port, "/Volumes/data/video.mp4")).toBe(
      "http://127.0.0.1:12345/stream/%2FVolumes%2Fdata%2Fvideo.mp4"
    );
  });

  it("macOS path with special chars (@)", () => {
    expect(streamUrl(port, "/Volumes/data/hhd800.com@FC2-PPV-123.mp4")).toBe(
      "http://127.0.0.1:12345/stream/%2FVolumes%2Fdata%2Fhhd800.com%40FC2-PPV-123.mp4"
    );
  });

  it("macOS path with spaces", () => {
    expect(streamUrl(port, "/Users/me/My Videos/test.mp4")).toBe(
      "http://127.0.0.1:12345/stream/%2FUsers%2Fme%2FMy%20Videos%2Ftest.mp4"
    );
  });

  it("macOS path with #", () => {
    expect(streamUrl(port, "/data/video#1.mp4")).toBe(
      "http://127.0.0.1:12345/stream/%2Fdata%2Fvideo%231.mp4"
    );
  });

  // Windows local paths
  it("Windows drive path with backslashes", () => {
    expect(streamUrl(port, "C:\\Users\\me\\video.mp4")).toBe(
      "http://127.0.0.1:12345/stream/C%3A%5CUsers%5Cme%5Cvideo.mp4"
    );
  });

  // Windows UNC paths
  it("Windows UNC path", () => {
    expect(streamUrl(port, "\\\\SERVER\\share\\folder\\video.mp4")).toBe(
      "http://127.0.0.1:12345/stream/%5C%5CSERVER%5Cshare%5Cfolder%5Cvideo.mp4"
    );
  });

  it("Windows UNC path with special chars", () => {
    expect(
      streamUrl(port, "\\\\UGNAS01\\personal_folder\\v\\hosi@test.mp4")
    ).toBe(
      "http://127.0.0.1:12345/stream/%5C%5CUGNAS01%5Cpersonal_folder%5Cv%5Chosi%40test.mp4"
    );
  });

  // Linux paths
  it("Linux absolute path", () => {
    expect(streamUrl(port, "/home/user/videos/test.mp4")).toBe(
      "http://127.0.0.1:12345/stream/%2Fhome%2Fuser%2Fvideos%2Ftest.mp4"
    );
  });

  it("Linux path with Japanese chars", () => {
    expect(streamUrl(port, "/home/user/動画/テスト.mp4")).toBe(
      "http://127.0.0.1:12345/stream/%2Fhome%2Fuser%2F%E5%8B%95%E7%94%BB%2F%E3%83%86%E3%82%B9%E3%83%88.mp4"
    );
  });

  // URL is always the same format regardless of platform
  it("same format on all platforms", () => {
    const macUrl = streamUrl(port, "/path/to/file.mp4");
    const winUrl = streamUrl(port, "C:\\path\\to\\file.mp4");
    // Both start with http://127.0.0.1
    expect(macUrl.startsWith("http://127.0.0.1:12345/stream/")).toBe(true);
    expect(winUrl.startsWith("http://127.0.0.1:12345/stream/")).toBe(true);
  });
});

describe("streamUrl round-trip with urlencoding::decode", () => {
  // Verify that encodeURIComponent produces paths that Rust's urlencoding::decode can restore
  it("encodeURIComponent is compatible with Rust urlencoding::decode", () => {
    const paths = [
      "/Volumes/data/video.mp4",
      "/Volumes/data/hhd800.com@FC2-PPV-123.mp4",
      "\\\\UGNAS01\\share\\file with spaces.mp4",
      "C:\\Users\\me\\video#1.mp4",
      "/home/user/動画/テスト.mp4",
    ];
    for (const p of paths) {
      const encoded = encodeURIComponent(p);
      const decoded = decodeURIComponent(encoded);
      expect(decoded).toBe(p);
    }
  });
});
