use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let raydium_pool = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";

    println!("🔍 Finding Raydium V4 vault addresses...");
    println!("Pool: {}", raydium_pool);

    if let Ok(account) = client.get_account(&Pubkey::from_str(raydium_pool)?) {
        let data = account.data;
        println!("Data size: {} bytes", data.len());
        
        // Based on our analysis, let's look for specific patterns
        println!("\n📊 Analyzing known structure:");
        
        // Parse known fields from first 100 bytes
        if data.len() >= 100 {
            let nonce = u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]);
            let max_order = u64::from_le_bytes([data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23]]);
            let depth = u64::from_le_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]);
            let base_decimal = u64::from_le_bytes([data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39]]);
            let quote_decimal = u64::from_le_bytes([data[40], data[41], data[42], data[43], data[44], data[45], data[46], data[47]]);
            let state = u64::from_le_bytes([data[48], data[49], data[50], data[51], data[52], data[53], data[54], data[55]]);
            let min_size = u64::from_le_bytes([data[56], data[57], data[58], data[59], data[60], data[61], data[62], data[63]]);
            let vol_max_cut_ratio = u64::from_le_bytes([data[64], data[65], data[66], data[67], data[68], data[69], data[70], data[71]]);
            
            println!("  Nonce: {}", nonce);
            println!("  MaxOrder: {}", max_order);
            println!("  Depth: {}", depth);
            println!("  BaseDecimal: {}", base_decimal);
            println!("  QuoteDecimal: {}", quote_decimal);
            println!("  State: {}", state);
            println!("  MinSize: {}", min_size);
            println!("  VolMaxCutRatio: {}", vol_max_cut_ratio);
        }
        
        // Look for Serum/OpenBook program ID (we found it at position 560)
        println!("\n🔍 Looking for Serum/OpenBook related data:");
        if data.len() > 560 + 32 {
            if let Ok(serum_program) = Pubkey::try_from(&data[560..560+32]) {
                println!("  Serum Program ID at 560: {}", serum_program);
                
                // Look for OpenOrders account around this position
                for i in 540..600 {
                    if i + 32 <= data.len() {
                        if let Ok(pubkey) = Pubkey::try_from(&data[i..i+32]) {
                            let pubkey_str = pubkey.to_string();
                            
                            // Skip known addresses
                            if pubkey_str == "So11111111111111111111111111111111111111112" ||
                               pubkey_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" ||
                               pubkey_str == "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX" {
                                continue;
                            }
                            
                            // Check if this might be OpenOrders
                            if pubkey_str.len() == 44 && !pubkey_str.contains("11111111111111111111111111111111") {
                                println!("    Position {}: {} (Potential OpenOrders)", i, pubkey_str);
                                
                                // Try to get this account
                                if let Ok(acc) = client.get_account(&pubkey) {
                                    println!("      ✅ Account exists, size: {} bytes", acc.data.len());
                                } else {
                                    println!("      ❌ Account not found");
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Look for vault addresses in different positions
        println!("\n🔍 Looking for vault addresses in different positions:");
        
        // Try positions that might contain vault addresses
        let potential_vault_positions = vec![
            200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215,
            216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230, 231,
            232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247,
            248, 249, 250, 251, 252, 253, 254, 255, 256, 257, 258, 259, 260, 261, 262, 263,
            264, 265, 266, 267, 268, 269, 270, 271, 272, 273, 274, 275, 276, 277, 278, 279,
            280, 281, 282, 283, 284, 285, 286, 287, 288, 289, 290, 291, 292, 293, 294, 295,
            296, 297, 298, 299, 300, 301, 302, 303, 304, 305, 306, 307, 308, 309, 310, 311,
            312, 313, 314, 315, 316, 317, 318, 319, 320, 321, 322, 323, 324, 325, 326, 327,
            328, 329, 330, 331, 332, 333, 334, 335, 336, 337, 338, 339, 340, 341, 342, 343,
            344, 345, 346, 347, 348, 349, 350, 351, 352, 353, 354, 355, 356, 357, 358, 359,
            360, 361, 362, 363, 364, 365, 366, 367, 368, 369, 370, 371, 372, 373, 374, 375,
            376, 377, 378, 379, 380, 381, 382, 383, 384, 385, 386, 387, 388, 389, 390, 391,
            392, 393, 394, 395, 396, 397, 398, 399, 400, 401, 402, 403, 404, 405, 406, 407,
            408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 419, 420, 421, 422, 423,
            424, 425, 426, 427, 428, 429, 430, 431, 432, 433, 434, 435, 436, 437, 438, 439,
            440, 441, 442, 443, 444, 445, 446, 447, 448, 449, 450, 451, 452, 453, 454, 455,
            456, 457, 458, 459, 460, 461, 462, 463, 464, 465, 466, 467, 468, 469, 470, 471,
            472, 473, 474, 475, 476, 477, 478, 479, 480, 481, 482, 483, 484, 485, 486, 487,
            488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 500, 501, 502, 503,
            504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 515, 516, 517, 518, 519,
            520, 521, 522, 523, 524, 525, 526, 527, 528, 529, 530, 531, 532, 533, 534, 535,
            536, 537, 538, 539, 540, 541, 542, 543, 544, 545, 546, 547, 548, 549, 550, 551,
            552, 553, 554, 555, 556, 557, 558, 559, 560, 561, 562, 563, 564, 565, 566, 567,
            568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578, 579, 580, 581, 582, 583,
            584, 585, 586, 587, 588, 589, 590, 591, 592, 593, 594, 595, 596, 597, 598, 599,
            600, 601, 602, 603, 604, 605, 606, 607, 608, 609, 610, 611, 612, 613, 614, 615,
            616, 617, 618, 619, 620, 621, 622, 623, 624, 625, 626, 627, 628, 629, 630, 631,
            632, 633, 634, 635, 636, 637, 638, 639, 640, 641, 642, 643, 644, 645, 646, 647,
            648, 649, 650, 651, 652, 653, 654, 655, 656, 657, 658, 659, 660, 661, 662, 663,
            664, 665, 666, 667, 668, 669, 670, 671, 672, 673, 674, 675, 676, 677, 678, 679,
            680, 681, 682, 683, 684, 685, 686, 687, 688, 689, 690, 691, 692, 693, 694, 695,
            696, 697, 698, 699, 700, 701, 702, 703, 704, 705, 706, 707, 708, 709, 710, 711,
            712, 713, 714, 715, 716, 717, 718, 719, 720, 721, 722, 723, 724, 725, 726, 727,
            728, 729, 730, 731, 732, 733, 734, 735, 736, 737, 738, 739, 740, 741, 742, 743,
            744, 745, 746, 747, 748, 749, 750, 751
        ];
        
        for &pos in &potential_vault_positions {
            if pos + 32 <= data.len() {
                if let Ok(pubkey) = Pubkey::try_from(&data[pos..pos+32]) {
                    let pubkey_str = pubkey.to_string();
                    
                    // Skip known addresses
                    if pubkey_str == "So11111111111111111111111111111111111111112" ||
                       pubkey_str == "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" ||
                       pubkey_str == "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX" {
                        continue;
                    }
                    
                    // Check if this might be a vault
                    if pubkey_str.len() == 44 && !pubkey_str.contains("11111111111111111111111111111111") {
                        // Try to get this account
                        if let Ok(acc) = client.get_account(&pubkey) {
                            println!("  Position {}: {} (Account exists, size: {} bytes)", pos, pubkey_str, acc.data.len());
                            
                            // If it's a token account, it might be a vault
                            if acc.data.len() >= 72 {
                                // Check if it looks like a token account
                                let balance = u64::from_le_bytes([
                                    acc.data[64], acc.data[65], acc.data[66], acc.data[67],
                                    acc.data[68], acc.data[69], acc.data[70], acc.data[71]
                                ]);
                                println!("    Token balance: {}", balance);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
