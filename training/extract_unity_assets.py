#!/usr/bin/env python3
"""
FGO Sheba - Unity Asset Extractor

FGO uses Unity for its game engine, and most assets are stored in Unity
AssetBundles. This script extracts images from Unity asset files.

Requirements:
    pip install UnityPy pillow tqdm

Usage:
    python extract_unity_assets.py --input ./assets/raw --output ./extracted
    python extract_unity_assets.py --apk ./fgo.apk --output ./extracted
"""

import argparse
import json
import os
import re
import sys
import tempfile
import zipfile
from pathlib import Path
from typing import Dict, List, Optional, Set

try:
    import UnityPy
    from UnityPy.enums import ClassIDType
    from PIL import Image
    from tqdm import tqdm
except ImportError:
    print("Missing dependencies. Install with:")
    print("  pip install UnityPy pillow tqdm")
    sys.exit(1)


# Asset type patterns for categorization
ASSET_PATTERNS = {
    "cards": [
        r"commandcard",
        r"card_.*_(buster|arts|quick)",
        r"command_card",
    ],
    "servants": [
        r"servant_",
        r"svt_",
        r"chara_",
        r"face_",
        r"status_servant",
    ],
    "enemies": [
        r"enemy_",
        r"mob_",
        r"monster_",
    ],
    "skills": [
        r"skill_",
        r"buff_",
        r"debuff_",
        r"skillicon",
    ],
    "class_icons": [
        r"classicon",
        r"class_",
        r"icon_class",
    ],
    "np": [
        r"noble_",
        r"np_",
        r"phantasm",
    ],
    "ui": [
        r"btn_",
        r"button_",
        r"ui_",
        r"frame_",
        r"window_",
        r"dialog_",
    ],
    "backgrounds": [
        r"bg_",
        r"background_",
        r"battle_bg",
        r"field_",
    ],
}


class UnityAssetExtractor:
    """Extracts images from Unity asset bundles."""

    def __init__(self, output_dir: Path):
        self.output_dir = output_dir
        self.stats = {
            "total_files": 0,
            "extracted_images": 0,
            "categories": {},
        }

    def extract_from_apk(self, apk_path: Path) -> bool:
        """Extract Unity assets from an APK file."""
        print(f"Extracting Unity assets from APK: {apk_path}")

        try:
            with tempfile.TemporaryDirectory() as temp_dir:
                temp_path = Path(temp_dir)

                # Extract APK
                print("Extracting APK contents...")
                with zipfile.ZipFile(apk_path, "r") as zf:
                    # Find asset files
                    asset_files = [
                        f for f in zf.namelist()
                        if f.startswith("assets/") and (
                            f.endswith(".assets") or
                            f.endswith(".bundle") or
                            f.endswith(".ab") or
                            "assetbundle" in f.lower()
                        )
                    ]

                    print(f"Found {len(asset_files)} asset files")

                    # Extract asset files
                    for af in tqdm(asset_files, desc="Extracting"):
                        zf.extract(af, temp_path)

                # Process extracted assets
                assets_dir = temp_path / "assets"
                if assets_dir.exists():
                    self.extract_from_directory(assets_dir)

            return True

        except Exception as e:
            print(f"Extraction failed: {e}")
            import traceback
            traceback.print_exc()
            return False

    def extract_from_directory(self, directory: Path):
        """Extract images from all asset files in a directory."""
        print(f"\nScanning directory: {directory}")

        # Find all potential asset files
        asset_files = []
        for ext in ["*.assets", "*.bundle", "*.ab", "*"]:
            asset_files.extend(directory.rglob(ext))

        # Filter to actual Unity asset files
        valid_assets = []
        for af in asset_files:
            if af.is_file() and self._is_unity_asset(af):
                valid_assets.append(af)

        print(f"Found {len(valid_assets)} Unity asset files")
        self.stats["total_files"] = len(valid_assets)

        # Create output directories
        for category in ASSET_PATTERNS:
            (self.output_dir / category).mkdir(parents=True, exist_ok=True)
        (self.output_dir / "other").mkdir(parents=True, exist_ok=True)

        # Process each asset file
        for asset_path in tqdm(valid_assets, desc="Processing assets"):
            try:
                self._process_asset_file(asset_path)
            except Exception as e:
                print(f"\nFailed to process {asset_path.name}: {e}")

        # Save statistics
        self._save_stats()

    def _is_unity_asset(self, file_path: Path) -> bool:
        """Check if file is a Unity asset by reading header."""
        try:
            with open(file_path, "rb") as f:
                header = f.read(32)

            # Unity asset signatures
            if header[:7] == b"UnityFS":
                return True
            if header[:8] == b"UnityWeb":
                return True
            if header[:11] == b"UnityRaw":
                return True

            # Also check for serialized file format
            # (older Unity format starts differently)
            if len(header) >= 4:
                # Check for valid file size header
                import struct
                if header[0:4] != b"\x00\x00\x00\x00":
                    return True

            return False
        except:
            return False

    def _process_asset_file(self, asset_path: Path):
        """Process a single Unity asset file."""
        try:
            env = UnityPy.load(str(asset_path))

            for obj in env.objects:
                if obj.type in [ClassIDType.Texture2D, ClassIDType.Sprite]:
                    try:
                        data = obj.read()

                        # Get image
                        if hasattr(data, "image"):
                            image = data.image

                            # Get name
                            name = getattr(data, "m_Name", None) or getattr(data, "name", None)
                            if not name:
                                name = f"texture_{obj.path_id}"

                            # Categorize and save
                            self._save_image(image, name)

                    except Exception as e:
                        pass  # Skip problematic textures

        except Exception as e:
            pass  # Skip files that can't be loaded

    def _save_image(self, image: Image.Image, name: str):
        """Save an image to the appropriate category directory."""
        # Clean name
        safe_name = re.sub(r"[^\w\-_.]", "_", name)
        if not safe_name.endswith(".png"):
            safe_name += ".png"

        # Determine category
        category = self._categorize_asset(name)

        # Track stats
        if category not in self.stats["categories"]:
            self.stats["categories"][category] = 0
        self.stats["categories"][category] += 1
        self.stats["extracted_images"] += 1

        # Save image
        output_path = self.output_dir / category / safe_name

        # Handle duplicates
        counter = 1
        while output_path.exists():
            output_path = self.output_dir / category / f"{Path(safe_name).stem}_{counter}.png"
            counter += 1

        try:
            # Convert to RGB if necessary
            if image.mode in ("RGBA", "LA") or (
                image.mode == "P" and "transparency" in image.info
            ):
                image.save(output_path, "PNG")
            else:
                image = image.convert("RGB")
                image.save(output_path, "PNG")
        except Exception as e:
            print(f"\nFailed to save {safe_name}: {e}")

    def _categorize_asset(self, name: str) -> str:
        """Categorize an asset based on its name."""
        name_lower = name.lower()

        for category, patterns in ASSET_PATTERNS.items():
            for pattern in patterns:
                if re.search(pattern, name_lower):
                    return category

        return "other"

    def _save_stats(self):
        """Save extraction statistics."""
        stats_path = self.output_dir / "extraction_stats.json"
        with open(stats_path, "w") as f:
            json.dump(self.stats, f, indent=2)

        print("\n" + "=" * 50)
        print("Extraction Statistics")
        print("=" * 50)
        print(f"Total asset files processed: {self.stats['total_files']}")
        print(f"Total images extracted: {self.stats['extracted_images']}")
        print("\nBy category:")
        for category, count in sorted(self.stats["categories"].items()):
            print(f"  {category}: {count}")


class CardDatasetBuilder:
    """Builds a training dataset from extracted card images."""

    def __init__(self, extracted_dir: Path, output_dir: Path):
        self.extracted_dir = extracted_dir
        self.output_dir = output_dir

    def build(self):
        """Build the card classification dataset."""
        print("\nBuilding card classification dataset...")

        # Create class directories
        classes = ["buster", "arts", "quick", "np", "unknown"]
        for cls in classes:
            (self.output_dir / cls).mkdir(parents=True, exist_ok=True)

        # Process card images
        cards_dir = self.extracted_dir / "cards"
        if not cards_dir.exists():
            print("No cards directory found")
            return

        card_files = list(cards_dir.glob("*.png"))
        print(f"Found {len(card_files)} card images")

        # Classify by analyzing filenames and colors
        for img_path in tqdm(card_files, desc="Classifying cards"):
            self._classify_card(img_path)

        # Generate info
        self._generate_info()

    def _classify_card(self, img_path: Path):
        """Classify a card image."""
        name = img_path.stem.lower()

        # Try to classify by filename first
        category = None
        if "buster" in name:
            category = "buster"
        elif "arts" in name:
            category = "arts"
        elif "quick" in name:
            category = "quick"
        elif "np" in name or "noble" in name or "phantasm" in name:
            category = "np"

        # Fall back to color analysis
        if not category:
            category = self._classify_by_color(img_path)

        # Copy to category directory
        dest = self.output_dir / category / img_path.name
        counter = 1
        while dest.exists():
            dest = self.output_dir / category / f"{img_path.stem}_{counter}.png"
            counter += 1

        try:
            import shutil
            shutil.copy2(img_path, dest)
        except Exception as e:
            print(f"Failed to copy {img_path}: {e}")

    def _classify_by_color(self, img_path: Path) -> str:
        """Classify card by dominant color."""
        try:
            with Image.open(img_path) as img:
                img = img.convert("RGB")

                # Get dominant color from center
                width, height = img.size
                cx, cy = width // 2, height // 2
                sample_size = min(width, height) // 4

                if sample_size < 1:
                    return "unknown"

                region = img.crop((
                    max(0, cx - sample_size),
                    max(0, cy - sample_size),
                    min(width, cx + sample_size),
                    min(height, cy + sample_size)
                ))

                pixels = list(region.getdata())
                if not pixels:
                    return "unknown"

                avg_r = sum(p[0] for p in pixels) / len(pixels)
                avg_g = sum(p[1] for p in pixels) / len(pixels)
                avg_b = sum(p[2] for p in pixels) / len(pixels)

                # Classify by color
                if avg_r > 180 and avg_r > avg_g + 30 and avg_r > avg_b + 30:
                    return "buster"
                elif avg_b > 180 and avg_b > avg_r + 30 and avg_b > avg_g + 30:
                    return "arts"
                elif avg_g > 180 and avg_g > avg_r + 30 and avg_g > avg_b + 30:
                    return "quick"
                else:
                    return "unknown"

        except Exception as e:
            return "unknown"

    def _generate_info(self):
        """Generate dataset information."""
        info = {"classes": {}, "total": 0}

        for cls_dir in self.output_dir.iterdir():
            if cls_dir.is_dir():
                count = len(list(cls_dir.glob("*.png")))
                info["classes"][cls_dir.name] = count
                info["total"] += count

        # Save info
        with open(self.output_dir / "dataset_info.json", "w") as f:
            json.dump(info, f, indent=2)

        print("\nDataset statistics:")
        for cls, count in sorted(info["classes"].items()):
            print(f"  {cls}: {count}")
        print(f"  Total: {info['total']}")


def main():
    parser = argparse.ArgumentParser(
        description="Extract images from FGO Unity assets"
    )
    parser.add_argument(
        "--apk",
        type=str,
        help="Path to APK file",
    )
    parser.add_argument(
        "--input",
        type=str,
        help="Path to directory containing Unity asset files",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="./unity_extracted",
        help="Output directory for extracted images",
    )
    parser.add_argument(
        "--build-dataset",
        action="store_true",
        help="Build training dataset from extracted images",
    )
    args = parser.parse_args()

    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    extractor = UnityAssetExtractor(output_dir)

    if args.apk:
        apk_path = Path(args.apk)
        if not apk_path.exists():
            print(f"APK not found: {apk_path}")
            sys.exit(1)
        extractor.extract_from_apk(apk_path)

    elif args.input:
        input_dir = Path(args.input)
        if not input_dir.exists():
            print(f"Input directory not found: {input_dir}")
            sys.exit(1)
        extractor.extract_from_directory(input_dir)

    else:
        print("Please specify --apk or --input")
        sys.exit(1)

    # Build training dataset if requested
    if args.build_dataset:
        dataset_dir = output_dir.parent / "card_dataset"
        builder = CardDatasetBuilder(output_dir, dataset_dir)
        builder.build()

    print(f"\nExtracted images saved to: {output_dir}")


if __name__ == "__main__":
    main()
