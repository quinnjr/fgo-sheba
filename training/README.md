# FGO Sheba - Training Data & Model Training

This directory contains scripts for downloading game assets and training ML models for FGO automation.

## Overview

FGO Sheba uses machine learning models for:
- **Card Classification**: Identifying Buster/Arts/Quick/NP cards
- **Servant Recognition**: Matching cards to servants
- **UI State Detection**: Recognizing current game screen
- **Enemy Detection**: Locating and classifying enemies

## Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Option 1: Download and extract APK assets
python download_assets.py --region na --output ./training_data --prepare-training

# Option 2: Extract from existing APK
python download_assets.py --apk /path/to/fgo.apk --output ./training_data --prepare-training

# Option 3: Extract Unity assets (more thorough)
python extract_unity_assets.py --apk /path/to/fgo.apk --output ./unity_extracted --build-dataset

# Train the card classifier
python train_card_classifier.py --data_dir ./training_data/card_dataset --output ../models/card_classifier.onnx
```

## Scripts

### `download_assets.py`

Downloads FGO APK from APKPure/APKMirror and extracts image assets.

```bash
# Download NA version
python download_assets.py --region na --output ./training_data

# Download JP version
python download_assets.py --region jp --output ./training_data

# Use existing APK
python download_assets.py --apk ./fgo.apk --output ./training_data

# Skip download, just prepare training data
python download_assets.py --skip-download --output ./training_data --prepare-training
```

### `extract_unity_assets.py`

Extracts images from Unity AssetBundles (FGO's actual asset format).

```bash
# Extract from APK
python extract_unity_assets.py --apk ./fgo.apk --output ./unity_extracted

# Extract from directory of asset files
python extract_unity_assets.py --input ./assets --output ./unity_extracted

# Also build training dataset
python extract_unity_assets.py --apk ./fgo.apk --output ./unity_extracted --build-dataset
```

### `train_card_classifier.py`

Trains a CNN model to classify command cards.

```bash
# Basic training
python train_card_classifier.py --data_dir ./card_dataset --output card_classifier.onnx

# With custom parameters
python train_card_classifier.py \
    --data_dir ./card_dataset \
    --output ../models/card_classifier.onnx \
    --epochs 100 \
    --batch_size 64 \
    --lr 0.0001
```

## Dataset Structure

The training scripts expect data in this structure:

```
training_data/
├── card_dataset/
│   ├── buster/          # Red Buster cards
│   │   ├── card_001.png
│   │   └── ...
│   ├── arts/            # Blue Arts cards
│   │   └── ...
│   ├── quick/           # Green Quick cards
│   │   └── ...
│   ├── np/              # Noble Phantasm cards
│   │   └── ...
│   └── unknown/         # Unclassified
│       └── ...
├── servant_portraits/   # Servant face images
│   └── ...
└── ui_templates/        # UI element templates
    ├── buttons/
    ├── frames/
    └── icons/
```

## Manual Data Collection

For best results, supplement APK assets with screenshots:

1. **Battle Screenshots**: Capture screenshots during card selection
2. **Crop Cards**: Extract individual cards (256x380 px typical)
3. **Label by Type**: Sort into buster/arts/quick/np folders
4. **Augment Data**: The training script applies augmentation automatically

### Recommended Amounts

| Asset Type | Minimum | Recommended |
|------------|---------|-------------|
| Cards (per class) | 100 | 500+ |
| Servant portraits | 50 | 200+ |
| UI templates | 20 | 50+ |

## Model Architecture

### Card Classifier

- Input: 150x200 RGB image
- Output: 5 classes (Buster, Arts, Quick, NP, Unknown)
- Architecture: Simple CNN with 4 conv layers
- Export: ONNX format for cross-platform inference

### Servant Matcher (Future)

- Input: 128x128 RGB portrait
- Output: Embedding vector for similarity matching
- Architecture: ResNet-18 based encoder

## Troubleshooting

### APK Download Fails

If automatic download fails:
1. Visit APKPure or APKMirror manually
2. Download the FGO APK
3. Use `--apk` flag with the downloaded file

### Unity Asset Extraction Issues

If UnityPy fails:
1. Try AssetStudio (GUI tool): https://github.com/Perfare/AssetStudio
2. Export textures manually to PNG
3. Organize into the dataset structure

### Low Model Accuracy

1. Check class balance in dataset
2. Add more training data
3. Increase epochs
4. Try data augmentation settings

## Output Models

Trained models are saved to `../models/`:

- `card_classifier.onnx` - Card type classification
- `servant_detector.onnx` - Servant/enemy detection (future)
- `ui_state.onnx` - UI screen classification (future)

These ONNX models are loaded by the Rust `sheba-vision` module.
