![Chroniq-Open Logo](tmp_file/logio.jpg)

# Chroniq-Open

This is the README file for the Chroniq-Open project. Please provide details about the project, its purpose, and how to use it.

## About Chroniqapp

### Chroniqapp: Redefining Memory Storage with a Decentralized Album and a New Social Experience

In the digital age, every highlight deserves to be preserved forever. Chroniqapp was created to make that vision a reality — revolutionizing how we capture, preserve, and share our most treasured memories.

### Built on Solana: A Future-Ready Foundation

Chroniqapp is powered by Solana, the world’s fastest and most efficient public blockchain. Leveraging Solana’s unmatched speed, ultra-low latency, and minimal gas fees, Chroniqapp offers a decentralized, high-performance memory storage solution.

Every photo and video stored with Chroniqapp becomes a permanent, immutable record on the blockchain. Unlike traditional albums that rely on local drives or centralized cloud services — prone to data loss, privacy risks, and access limitations — Chroniqapp empowers users with true ownership and enduring security.

With just a minimal gas fee, your most precious moments can live on-chain forever, safe from server outages or platform shutdowns.

Traditional digital albums are vulnerable. Chroniqapp embraces the Web3.0 revolution, building the world’s first truly decentralized memory platform — delivering immutability, permanence, and complete data sovereignty at a fraction of the traditional cost.

### Creative Templates, Filters, and Layout Designs

### A Canvas for Your Memories + Seamless Social Sharing

Chroniqapp is more than a storage solution — it's a gallery for your life's milestones. With a variety of artistic templates and creative filters, users can elevate their memories into lasting digital artworks, all securely stored on-chain. Key features include:

- Share your highlights as beautifully crafted NFTs across social platforms;
- Invite friends to view, like, and comment, turning memories into shared experiences;
- Collaborate with others on group albums — perfect for trips, events, and collective memories.

Chroniqapp transforms memories into timeless digital treasures — owned by you, forever.

## Getting Started

### Prerequisites and Startup Steps

#### Deploy ComfyUI
   ```
   git clone git@github.com:comfyanonymous/ComfyUI.git
   ```

#### Download Model Files
   **Note**: You need to apply for access to the models before you can download them.
   ```
   cd ComfyUI/models/clip
   wget https://huggingface.co/stabilityai/stable-diffusion-3.5-large-turbo/blob/main/text_encoders/clip_g.safetensors
   wget https://huggingface.co/stabilityai/stable-diffusion-3.5-large-turbo/blob/main/text_encoders/clip_l.safetensors
   wget https://huggingface.co/stabilityai/stable-diffusion-3.5-large-turbo/blob/main/text_encoders/t5xxl_fp8_e4m3fn.safetensors
   cd ComfyUI/models/checkpoints
   wget https://huggingface.co/stabilityai/stable-diffusion-3.5-large-turbo/blob/main/sd3.5_large_turbo.safetensors
   ```

#### Start ComfyUI
   ```
   cd ComfyUI
   python main.py
   ```

#### Start Chroniq-Open
   **Note**: Before starting, ensure the following configurations are set in the `.env` file:
   - **SD3 Model File Configuration**:
     ```
     SD3_BASE_SERVER=127.0.0.1:8188
     SD3_MODEL_FILE_NAME=sd3.5_large_turbo.safetensors
     SD3_CLIP_NAME1=clip_g.safetensors
     SD3_CLIP_NAME2=clip_l.safetensors
     SD3_CLIP_NAME3=t5xxl_fp8_e4m3fn.safetensors
     ```
   - **Local File Access Configuration**:
     ```
     IMG_TMP_POINT=http://127.0.0.1:8000/file
     IMG_TEMP_PATH=tmp_file
     ```
   ```
   cd chroniq-open
   cargo run
   ```

## License

Specify the license under which the project is distributed.
