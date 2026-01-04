#!/usr/bin/env python3
"""
FGO Sheba - Asset Downloader and Training Data Preparer

This script downloads FGO assets from Atlas Academy API (legal, public game data)
and organizes them for AI model training.

Usage:
    python download_assets.py --region na --output ./training_data
    python download_assets.py --apk /path/to/existing.apk --output ./training_data
"""

import argparse
import json
import os
import re
import shutil
import sys
import tempfile
import zipfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Dict, List, Optional
from urllib.parse import urljoin

try:
    import requests
    from PIL import Image
    from tqdm import tqdm
except ImportError:
    print("Missing dependencies. Install with:")
    print("  pip install requests pillow tqdm")
    sys.exit(1)


# Atlas Academy API endpoints
ATLAS_API = "https://api.atlasacademy.io"
ATLAS_EXPORTS = {
    "na": f"{ATLAS_API}/export/NA",
    "jp": f"{ATLAS_API}/export/JP",
}

# Asset CDN URLs
ASSET_CDN = {
    "na": "https://static.atlasacademy.io/NA",
    "jp": "https://static.atlasacademy.io/JP",
}

# Card color mappings
CARD_TYPES = {
    1: "arts",
    2: "buster",
    3: "quick",
}

# Class names for icons
CLASS_NAMES = {
    1: "saber",
    2: "archer",
    3: "lancer",
    4: "rider",
    5: "caster",
    6: "assassin",
    7: "berserker",
    8: "shielder",
    9: "ruler",
    10: "avenger",
    11: "moon_cancer",
    12: "alter_ego",
    13: "foreigner",
    14: "pretender",
    15: "beast",
}

# User agent
USER_AGENT = (
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
)


class AtlasDownloader:
    """Downloads FGO assets from Atlas Academy API and CDN."""

    def __init__(self, region: str = "na"):
        self.region = region.upper()
        self.api_base = ATLAS_API
        self.export_base = ATLAS_EXPORTS.get(region, ATLAS_EXPORTS["na"])
        self.cdn_base = ASSET_CDN.get(region, ASSET_CDN["na"])
        self.session = requests.Session()
        self.session.headers.update({"User-Agent": USER_AGENT})

    def get_servant_list(self) -> List[Dict]:
        """Get list of all servants."""
        print("Fetching servant data...")
        url = f"{self.export_base}/basic_servant.json"
        response = self.session.get(url)
        response.raise_for_status()
        return response.json()

    def get_servant_details(self, servant_id: int) -> Dict:
        """Get detailed servant data."""
        url = f"{self.api_base}/nice/{self.region}/servant/{servant_id}"
        response = self.session.get(url)
        response.raise_for_status()
        return response.json()

    def get_enemy_list(self) -> List[Dict]:
        """Get list of enemies."""
        print("Fetching enemy data...")
        url = f"{self.export_base}/basic_servant.json"  # Enemies are also in servant data
        response = self.session.get(url)
        response.raise_for_status()
        # Filter for enemies (usually those with collectionNo > 300 or special flags)
        data = response.json()
        return [s for s in data if s.get("collectionNo", 0) > 300 or "enemy" in s.get("name", "").lower()]

    def get_craft_essence_list(self) -> List[Dict]:
        """Get list of craft essences."""
        print("Fetching craft essence data...")
        url = f"{self.export_base}/basic_equip.json"
        response = self.session.get(url)
        response.raise_for_status()
        return response.json()

    def download_image(self, url: str, output_path: Path) -> bool:
        """Download a single image."""
        try:
            response = self.session.get(url, timeout=30)
            response.raise_for_status()
            output_path.parent.mkdir(parents=True, exist_ok=True)
            with open(output_path, "wb") as f:
                f.write(response.content)
            return True
        except Exception as e:
            return False

    def download_servant_assets(self, output_dir: Path, limit: int = None):
        """Download servant face/portrait assets."""
        print("\n=== Downloading Servant Assets ===")
        servants = self.get_servant_list()

        if limit:
            servants = servants[:limit]

        face_dir = output_dir / "servants" / "faces"
        portrait_dir = output_dir / "servants" / "portraits"
        face_dir.mkdir(parents=True, exist_ok=True)
        portrait_dir.mkdir(parents=True, exist_ok=True)

        tasks = []
        for servant in servants:
            svt_id = servant.get("id")
            coll_no = servant.get("collectionNo", 0)

            # Face icon (used in battle party display)
            face_url = f"{self.cdn_base}/Faces/f_{svt_id}0.png"
            face_path = face_dir / f"{coll_no}_{svt_id}_face.png"
            tasks.append((face_url, face_path, f"face_{svt_id}"))

            # Portrait (used in servant info)
            portrait_url = f"{self.cdn_base}/Faces/f_{svt_id}1.png"
            portrait_path = portrait_dir / f"{coll_no}_{svt_id}_portrait.png"
            tasks.append((portrait_url, portrait_path, f"portrait_{svt_id}"))

        print(f"Downloading {len(tasks)} servant images...")
        self._download_batch(tasks)

    def download_card_assets(self, output_dir: Path, limit: int = None):
        """Download command card assets."""
        print("\n=== Downloading Command Card Assets ===")
        servants = self.get_servant_list()

        if limit:
            servants = servants[:limit]

        card_dir = output_dir / "cards"
        for card_type in ["arts", "buster", "quick"]:
            (card_dir / card_type).mkdir(parents=True, exist_ok=True)

        tasks = []
        for servant in servants:
            try:
                svt_id = servant.get("id")
                details = self.get_servant_details(svt_id)

                cards = details.get("cards", [])
                for idx, card_type_id in enumerate(cards):
                    card_type = CARD_TYPES.get(card_type_id, "unknown")
                    if card_type == "unknown":
                        continue

                    # Command cards have specific asset paths
                    card_url = f"{self.cdn_base}/Commands/cmd_card_{card_type}_{svt_id}.png"
                    card_path = card_dir / card_type / f"{svt_id}_card{idx+1}.png"
                    tasks.append((card_url, card_path, f"card_{svt_id}_{idx}"))

            except Exception as e:
                continue

        print(f"Downloading {len(tasks)} card images...")
        self._download_batch(tasks)

    def download_skill_icons(self, output_dir: Path):
        """Download skill icons."""
        print("\n=== Downloading Skill Icons ===")

        skill_dir = output_dir / "skills"
        skill_dir.mkdir(parents=True, exist_ok=True)

        # Download common skill icons (numbered 1-999)
        tasks = []
        for skill_id in range(1, 500):  # Most skills are in this range
            url = f"{self.cdn_base}/SkillIcons/skill_{str(skill_id).zfill(5)}.png"
            path = skill_dir / f"skill_{skill_id}.png"
            tasks.append((url, path, f"skill_{skill_id}"))

        print(f"Attempting to download skill icons...")
        self._download_batch(tasks, show_errors=False)

    def download_class_icons(self, output_dir: Path):
        """Download class icons."""
        print("\n=== Downloading Class Icons ===")

        class_dir = output_dir / "class_icons"
        class_dir.mkdir(parents=True, exist_ok=True)

        tasks = []
        for class_id, class_name in CLASS_NAMES.items():
            # Various class icon sizes and variants
            for variant in ["", "_gold", "_silver", "_bronze"]:
                url = f"{self.cdn_base}/ClassIcons/class{variant}_{class_id}.png"
                path = class_dir / f"{class_name}{variant}.png"
                tasks.append((url, path, f"class_{class_name}{variant}"))

        print(f"Downloading {len(tasks)} class icons...")
        self._download_batch(tasks, show_errors=False)

    def download_ui_assets(self, output_dir: Path):
        """Download UI element assets."""
        print("\n=== Downloading UI Assets ===")

        ui_dir = output_dir / "ui"
        ui_dir.mkdir(parents=True, exist_ok=True)

        # Common UI elements
        ui_assets = [
            ("attack_btn", "Buttons/btn_attack.png"),
            ("skill_btn", "Buttons/btn_skill.png"),
            ("master_skill_btn", "Buttons/btn_master_skill.png"),
            ("np_gauge_frame", "Battle/np_gauge_frame.png"),
            ("hp_bar_frame", "Battle/hp_bar_frame.png"),
            ("battle_frame", "Battle/battle_frame.png"),
            ("card_frame_buster", "Cards/card_frame_buster.png"),
            ("card_frame_arts", "Cards/card_frame_arts.png"),
            ("card_frame_quick", "Cards/card_frame_quick.png"),
        ]

        tasks = []
        for name, asset_path in ui_assets:
            url = f"{self.cdn_base}/{asset_path}"
            path = ui_dir / f"{name}.png"
            tasks.append((url, path, name))

        self._download_batch(tasks, show_errors=False)

    def download_all(self, output_dir: Path, limit: int = None):
        """Download all asset categories."""
        output_dir.mkdir(parents=True, exist_ok=True)

        self.download_servant_assets(output_dir, limit)
        self.download_card_assets(output_dir, limit)
        self.download_skill_icons(output_dir)
        self.download_class_icons(output_dir)
        self.download_ui_assets(output_dir)

        # Save metadata
        self._save_metadata(output_dir)

    def _download_batch(self, tasks: List[tuple], show_errors: bool = True):
        """Download multiple files in parallel."""
        successful = 0
        failed = 0

        with ThreadPoolExecutor(max_workers=8) as executor:
            futures = {
                executor.submit(self.download_image, url, path): desc
                for url, path, desc in tasks
            }

            with tqdm(total=len(tasks), desc="Downloading") as pbar:
                for future in as_completed(futures):
                    if future.result():
                        successful += 1
                    else:
                        failed += 1
                    pbar.update(1)

        print(f"  Downloaded: {successful}, Failed: {failed}")

    def _save_metadata(self, output_dir: Path):
        """Save download metadata."""
        metadata = {
            "region": self.region,
            "source": "Atlas Academy",
            "api_base": self.api_base,
            "cdn_base": self.cdn_base,
        }

        with open(output_dir / "metadata.json", "w") as f:
            json.dump(metadata, f, indent=2)


class APKExtractor:
    """Extracts assets from FGO APK file."""

    ASSET_CATEGORIES = {
        "cards": ["card", "command", "cmd_"],
        "servants": ["servant", "svt", "chara", "face"],
        "enemies": ["enemy", "mob"],
        "ui": ["btn", "button", "ui", "icon", "frame"],
        "skills": ["skill", "buff", "debuff"],
        "class_icons": ["class", "classicon"],
        "backgrounds": ["bg", "background", "battle"],
    }

    def __init__(self, apk_path: Path):
        self.apk_path = apk_path
        self.temp_dir = None

    def extract(self, output_dir: Path) -> bool:
        """Extract APK contents."""
        print(f"\nExtracting APK: {self.apk_path}")

        try:
            self.temp_dir = Path(tempfile.mkdtemp())

            with zipfile.ZipFile(self.apk_path, "r") as zf:
                file_list = zf.namelist()
                print(f"Found {len(file_list)} files in APK")

                for file in tqdm(file_list, desc="Extracting"):
                    try:
                        zf.extract(file, self.temp_dir)
                    except Exception:
                        pass

            self._process_assets(output_dir)
            return True

        except Exception as e:
            print(f"Extraction failed: {e}")
            return False

        finally:
            if self.temp_dir and self.temp_dir.exists():
                shutil.rmtree(self.temp_dir, ignore_errors=True)

    def _process_assets(self, output_dir: Path):
        """Process and organize extracted assets."""
        print("\nProcessing assets...")

        for category in self.ASSET_CATEGORIES:
            (output_dir / category).mkdir(parents=True, exist_ok=True)
        (output_dir / "unknown").mkdir(parents=True, exist_ok=True)

        assets_dir = self.temp_dir / "assets"
        res_dir = self.temp_dir / "res"

        if assets_dir.exists():
            self._scan_directory(assets_dir, output_dir)
        if res_dir.exists():
            self._scan_directory(res_dir, output_dir)

        print(f"\nAssets saved to: {output_dir}")

    def _scan_directory(self, directory: Path, output_dir: Path):
        """Scan directory for image assets."""
        image_extensions = {".png", ".jpg", ".jpeg", ".webp", ".bmp"}

        for file_path in directory.rglob("*"):
            if not file_path.is_file():
                continue

            suffix = file_path.suffix.lower()
            if suffix in image_extensions:
                self._categorize_image(file_path, output_dir)
            elif suffix == "" and self._is_image_data(file_path):
                self._categorize_image(file_path, output_dir, detect_format=True)

    def _is_image_data(self, file_path: Path) -> bool:
        """Check if file contains image data."""
        try:
            with open(file_path, "rb") as f:
                header = f.read(16)

            if header[:8] == b"\x89PNG\r\n\x1a\n":
                return True
            if header[:2] == b"\xff\xd8":
                return True
            if header[:4] == b"RIFF" and header[8:12] == b"WEBP":
                return True
            return False
        except:
            return False

    def _categorize_image(self, file_path: Path, output_dir: Path, detect_format: bool = False):
        """Categorize an image file."""
        filename = file_path.stem.lower()
        relative_path = str(file_path.relative_to(self.temp_dir)).lower()

        category = "unknown"
        for cat, keywords in self.ASSET_CATEGORIES.items():
            for keyword in keywords:
                if keyword in filename or keyword in relative_path:
                    category = cat
                    break
            if category != "unknown":
                break

        safe_name = re.sub(r"[^\w\-_.]", "_", file_path.name)
        if detect_format:
            try:
                with Image.open(file_path) as img:
                    fmt = img.format.lower() if img.format else "png"
                    safe_name = f"{safe_name}.{fmt}"
            except:
                safe_name = f"{safe_name}.png"

        output_path = output_dir / category / safe_name

        counter = 1
        while output_path.exists():
            stem = output_path.stem
            suffix = output_path.suffix
            output_path = output_dir / category / f"{stem}_{counter}{suffix}"
            counter += 1

        try:
            shutil.copy2(file_path, output_path)
        except Exception as e:
            print(f"Failed to copy {file_path}: {e}")


class TrainingDataPreparer:
    """Prepares extracted assets for model training."""

    def __init__(self, assets_dir: Path):
        self.assets_dir = assets_dir

    def prepare_card_dataset(self, output_dir: Path):
        """Prepare card images for classifier training."""
        print("\n=== Preparing Card Training Dataset ===")

        card_dir = self.assets_dir / "cards"
        if not card_dir.exists():
            print("No card assets found")
            return

        classes = ["buster", "arts", "quick", "unknown"]
        for cls in classes:
            (output_dir / cls).mkdir(parents=True, exist_ok=True)

        # If cards are already categorized, just copy them
        for card_type in ["arts", "buster", "quick"]:
            src_dir = card_dir / card_type
            if src_dir.exists():
                for img_path in src_dir.glob("*.png"):
                    try:
                        with Image.open(img_path) as img:
                            img = img.convert("RGB")
                            img = img.resize((224, 224), Image.Resampling.LANCZOS)
                            dest = output_dir / card_type / img_path.name
                            img.save(dest)
                    except Exception as e:
                        print(f"Failed to process {img_path}: {e}")

        # Process uncategorized cards
        uncategorized = list(card_dir.glob("*.png")) + list(card_dir.glob("*.jpg"))
        for img_path in tqdm(uncategorized, desc="Processing uncategorized cards"):
            self._classify_card_by_color(img_path, output_dir)

        self._generate_dataset_info(output_dir)

    def _classify_card_by_color(self, img_path: Path, output_dir: Path):
        """Classify card by dominant color."""
        try:
            with Image.open(img_path) as img:
                img = img.convert("RGB")

                # Sample center region
                width, height = img.size
                cx, cy = width // 2, height // 2
                sample_size = min(width, height) // 4

                region = img.crop((
                    max(0, cx - sample_size),
                    max(0, cy - sample_size),
                    min(width, cx + sample_size),
                    min(height, cy + sample_size)
                ))

                pixels = list(region.getdata())
                if not pixels:
                    return

                avg_r = sum(p[0] for p in pixels) / len(pixels)
                avg_g = sum(p[1] for p in pixels) / len(pixels)
                avg_b = sum(p[2] for p in pixels) / len(pixels)

                # Buster: Red dominant
                # Arts: Blue dominant
                # Quick: Green dominant
                if avg_r > 150 and avg_r > avg_g + 30 and avg_r > avg_b + 30:
                    category = "buster"
                elif avg_b > 150 and avg_b > avg_r + 30 and avg_b > avg_g + 30:
                    category = "arts"
                elif avg_g > 150 and avg_g > avg_r + 30 and avg_g > avg_b + 30:
                    category = "quick"
                else:
                    category = "unknown"

                # Resize and save
                img_resized = img.resize((224, 224), Image.Resampling.LANCZOS)
                dest = output_dir / category / img_path.name
                counter = 1
                while dest.exists():
                    dest = output_dir / category / f"{img_path.stem}_{counter}{img_path.suffix}"
                    counter += 1
                img_resized.save(dest)

        except Exception as e:
            pass

    def _generate_dataset_info(self, output_dir: Path):
        """Generate dataset statistics."""
        info = {"classes": {}, "total": 0}

        for cls_dir in output_dir.iterdir():
            if cls_dir.is_dir():
                count = len(list(cls_dir.glob("*.png"))) + len(list(cls_dir.glob("*.jpg")))
                info["classes"][cls_dir.name] = count
                info["total"] += count

        info_path = output_dir / "dataset_info.json"
        with open(info_path, "w") as f:
            json.dump(info, f, indent=2)

        print(f"\nDataset statistics:")
        for cls, count in sorted(info["classes"].items()):
            print(f"  {cls}: {count} images")
        print(f"  Total: {info['total']} images")

    def prepare_servant_dataset(self, output_dir: Path):
        """Prepare servant portrait dataset."""
        print("\n=== Preparing Servant Portrait Dataset ===")

        servant_dir = self.assets_dir / "servants"
        if not servant_dir.exists():
            print("No servant assets found")
            return

        output_dir.mkdir(parents=True, exist_ok=True)

        for subdir in ["faces", "portraits"]:
            src = servant_dir / subdir
            if src.exists():
                for img_path in tqdm(list(src.glob("*.png")), desc=f"Processing {subdir}"):
                    try:
                        with Image.open(img_path) as img:
                            img = img.convert("RGB")
                            img = img.resize((128, 128), Image.Resampling.LANCZOS)
                            (output_dir / subdir).mkdir(parents=True, exist_ok=True)
                            img.save(output_dir / subdir / img_path.name)
                    except Exception as e:
                        pass

    def prepare_class_icon_dataset(self, output_dir: Path):
        """Prepare class icon dataset for classification."""
        print("\n=== Preparing Class Icon Dataset ===")

        class_dir = self.assets_dir / "class_icons"
        if not class_dir.exists():
            print("No class icon assets found")
            return

        output_dir.mkdir(parents=True, exist_ok=True)

        for img_path in tqdm(list(class_dir.glob("*.png")), desc="Processing class icons"):
            try:
                with Image.open(img_path) as img:
                    img = img.convert("RGBA")
                    img = img.resize((64, 64), Image.Resampling.LANCZOS)
                    img.save(output_dir / img_path.name)
            except Exception as e:
                pass


def main():
    parser = argparse.ArgumentParser(
        description="Download FGO assets from Atlas Academy for AI training"
    )
    parser.add_argument(
        "--region",
        type=str,
        choices=["na", "jp"],
        default="na",
        help="Game region (na or jp)",
    )
    parser.add_argument(
        "--apk",
        type=str,
        help="Path to existing APK file (alternative to Atlas download)",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="./training_data",
        help="Output directory for assets",
    )
    parser.add_argument(
        "--limit",
        type=int,
        help="Limit number of servants to download (for testing)",
    )
    parser.add_argument(
        "--prepare-training",
        action="store_true",
        help="Prepare training datasets after download",
    )
    parser.add_argument(
        "--skip-download",
        action="store_true",
        help="Skip download, only prepare existing assets",
    )
    args = parser.parse_args()

    output_dir = Path(args.output)
    assets_dir = output_dir / "assets"

    # Download or extract assets
    if args.apk:
        apk_path = Path(args.apk)
        if not apk_path.exists():
            print(f"APK not found: {apk_path}")
            sys.exit(1)

        extractor = APKExtractor(apk_path)
        if not extractor.extract(assets_dir):
            print("Extraction failed")
            sys.exit(1)
    elif not args.skip_download:
        print(f"Downloading FGO ({args.region.upper()}) assets from Atlas Academy...")
        print("This is legal, public game data from https://atlasacademy.io/\n")

        downloader = AtlasDownloader(args.region)
        downloader.download_all(assets_dir, limit=args.limit)

    # Prepare training data
    if args.prepare_training or args.skip_download:
        if assets_dir.exists():
            preparer = TrainingDataPreparer(assets_dir)
            preparer.prepare_card_dataset(output_dir / "card_dataset")
            preparer.prepare_servant_dataset(output_dir / "servant_dataset")
            preparer.prepare_class_icon_dataset(output_dir / "class_dataset")

            print("\n" + "=" * 60)
            print("Asset download and preparation complete!")
            print("=" * 60)
            print(f"\nOutput directory: {output_dir}")
            print("\nDirectory structure:")
            print("  assets/           - Raw downloaded assets")
            print("  card_dataset/     - Training data for card classifier")
            print("  servant_dataset/  - Training data for servant recognition")
            print("  class_dataset/    - Training data for class icon classifier")
            print("\nNext steps:")
            print("1. Review the datasets and manually correct any misclassifications")
            print("2. Add more training data from gameplay screenshots if needed")
            print("3. Run the training script:")
            print(f"   python train_card_classifier.py --data_dir {output_dir}/card_dataset")
        else:
            print("No assets directory found. Run download first.")


if __name__ == "__main__":
    main()
