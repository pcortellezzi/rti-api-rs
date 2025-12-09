#[cfg(test)]
mod compliance {
    use regex::Regex;
    use std::fs;
    use std::collections::{HashSet, HashMap};
    use std::path::Path;
    use std::process::Command;

    struct ApiSpec {
        id: i32,
        name: String,
        direction: String, // "From Client" or "From Server"
    }

    fn extract_text_via_poppler(pdf_path: &Path) -> Option<String> {
        // Paths to check for pdftotext.exe
        // Using the path provided by the user and standard locations
        let paths_to_check = [
            r"C:\\Users\\cortep\\Applications\\poppler-25.12.0\\Library\\bin\\pdftotext.exe",
            r"C:\\Users\\cortep\\Applications\\poppler-25.12.0\\bin\\pdftotext.exe",
            "pdftotext", // PATH fallback
        ];

        for bin in paths_to_check {
            // Try to execute pdftotext -layout <pdf> -
            let output = Command::new(bin)
                .arg("-layout") // Maintain original physical layout (essential for tables)
                .arg(pdf_path)
                .arg("-") // Print to stdout
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    return String::from_utf8(out.stdout).ok();
                }
            }
        }
        
        eprintln!("Could not find or execute pdftotext.exe. Please install poppler or update the path.");
        None
    }

    fn extract_specs_from_pdf() -> Vec<ApiSpec> {
        let path = Path::new("src/proto/Reference_Guide.pdf");
        if !path.exists() {
            eprintln!("Skipping compliance test: Reference_Guide.pdf not found at {:?}", path);
            return vec![];
        }

        let text = match extract_text_via_poppler(path) {
            Some(t) => t,
            None => return vec![],
        };



        let mut specs = Vec::new();
        
        // Regex to find a line that contains an ID and Direction
        // Pattern: ... 123 ... From Client
        let re_id_and_dir = Regex::new(r"(\d{2,4})\s*(?:From\s+)?(C\s*lient|S\s*erver)").unwrap();
        
        // Regex for multi-line: Line N has ID, Line N+1 has Direction
        let re_id_only = Regex::new(r"^\s*(\d{2,4})\s*$").unwrap();
        let re_dir_only = Regex::new(r"(?i)^\s*(?:From\s+)?(C\s*lient|S\s*erver)\s*$").unwrap();
        
        // Regex for the specific "Reject 75" case: Name and ID on Line N, Direction on Line N+1
        // "Reject 75"
        let re_name_id_line = Regex::new(r"(?i)(.+)\s+(\d{2,4})\s*$").unwrap();


        let lines: Vec<String> = text.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        let mut current_name_fragments: Vec<String> = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = &lines[i];
            let l_lower = line.to_lowercase();

            // Filter out known noise lines and reset buffer
            if l_lower.contains("www.rithmic.com") ||
               l_lower.contains("r | protocol api") ||
               l_lower.contains("direction") ||
               l_lower.contains("templates specific to") ||
               l_lower.contains("templates shared across") ||
               l_lower.contains("template name")
               {
                current_name_fragments.clear(); 
                i += 1;
                continue;
            }

            // --- Strategy 1: Single Line (ID + Dir) ---
            if let Some(caps) = re_id_and_dir.captures(line) {
                if let Ok(id) = caps[1].parse::<i32>() {
                    if id >= 10 {
                        let direction_raw = caps[2].replace(" ", "");
                        let direction = format!("From {}", direction_raw);
                        
                        let match_start = caps.get(1).unwrap().start();
                        let name_part = line[0..match_start].trim().to_string();
                        
                        let mut final_name_parts = current_name_fragments.clone();
                        if !name_part.is_empty() {
                            final_name_parts.push(name_part);
                        }
                        
                        // Lookahead for trailing name parts (e.g. "Response" on next line)
                        // This handles cases like:
                        // "Rithmic System Info      17      From Server"
                        // "      Response"
                        let mut j = 1;
                        while i + j < lines.len() {
                            let next_l = &lines[i+j];
                            let next_l_lower = next_l.to_lowercase();
                            
                            // Check if this next line is the start of a new definition or noise
                            if re_id_and_dir.is_match(next_l) 
                                || re_id_only.is_match(next_l) 
                                || re_name_id_line.is_match(next_l)
                                || re_dir_only.is_match(next_l)
                                || next_l.trim().is_empty() 
                                || next_l_lower.contains("www.rithmic.com")
                                || next_l_lower.contains("r | protocol api")
                                || next_l_lower.contains("templates specific")
                                || next_l_lower.contains("template name")
                                || next_l_lower.contains("template id")
                            {
                                break;
                            }
                            
                            // It looks like a continuation of the name
                            final_name_parts.push(next_l.trim().to_string());
                            j += 1;
                        }

                        let final_name = final_name_parts.join(" ").trim().to_string();

                        if !final_name.is_empty() {
                            if !specs.iter().any(|s: &ApiSpec| s.id == id) {
                                specs.push(ApiSpec { id, name: final_name, direction });
                            }
                        }
                        current_name_fragments.clear();
                        i += j;
                        continue;
                    }
                }
            }

            // --- Strategy 2: Multi-line (Name+ID on line N, Dir on line N+1) ---
            if let Some(caps_name_id) = re_name_id_line.captures(line) {
                if i + 1 < lines.len() {
                    let next_line = &lines[i+1];
                    if let Some(caps_dir) = re_dir_only.captures(next_line) {
                        if let Ok(id) = caps_name_id[2].parse::<i32>() {
                            if id >= 10 {
                                let direction_raw = caps_dir[1].replace(" ", "");
                                let direction = format!("From {}", direction_raw);
                                
                                let name_part = caps_name_id[1].trim().to_string();
                                
                                let mut final_name_parts = current_name_fragments.clone();
                                if !name_part.is_empty() {
                                    final_name_parts.push(name_part);
                                }
                                let final_name = final_name_parts.join(" ").trim().to_string();

                                if !final_name.is_empty() {
                                    if !specs.iter().any(|s: &ApiSpec| s.id == id) {
                                        specs.push(ApiSpec { id, name: final_name, direction });
                                    }
                                }
                                current_name_fragments.clear();
                                i += 2;
                                continue;
                            }
                        }
                    }
                }
            }

            // --- Strategy 3: Multi-line (ID on line N, Dir on line N+1) ---
            if let Some(caps_id) = re_id_only.captures(line) {
                if i + 1 < lines.len() {
                    let next_line = &lines[i+1];
                    if let Some(caps_dir) = re_dir_only.captures(next_line) {
                        if let Ok(id) = caps_id[1].parse::<i32>() {
                            if id >= 10 {
                                let direction_raw = caps_dir[1].replace(" ", "");
                                let direction = format!("From {}", direction_raw);
                                let final_name = current_name_fragments.join(" ").trim().to_string();

                                if !final_name.is_empty() {
                                    if !specs.iter().any(|s: &ApiSpec| s.id == id) {
                                        specs.push(ApiSpec { id, name: final_name, direction });
                                    }
                                }
                                current_name_fragments.clear();
                                i += 2;
                                continue;
                            }
                        }
                    }
                }
            }

            current_name_fragments.push(line.to_string());
            i += 1;
        }

        specs.sort_by_key(|s| s.id);
        specs
    }

    fn get_implemented_requests() -> HashMap<i32, String> {
        let content = fs::read_to_string("src/api/sender_api.rs").expect("Failed to read sender_api.rs");
        let re = Regex::new(r"let\s+(?:mut\s+)?req\s*=\s*(\w+)\s*\{[\s\S]*?template_id:\s*(\d+)").unwrap();
        let mut map = HashMap::new();
        for caps in re.captures_iter(&content) {
            if let (Some(name), Some(id_str)) = (caps.get(1), caps.get(2)) {
                if let Ok(id) = id_str.as_str().parse::<i32>() {
                    map.insert(id, name.as_str().to_string());
                }
            }
        }
        map
    }

    fn get_implemented_responses() -> HashMap<i32, String> {
        let content = fs::read_to_string("src/api/receiver_api.rs").expect("Failed to read receiver_api.rs");
        let re = Regex::new(r"(\d+)\s*=>\s*\{\s*[\s\S]*?let\s+resp\s*=\s*(\w+)::decode").unwrap();
        let mut map = HashMap::new();
        for caps in re.captures_iter(&content) {
            if let (Some(id_str), Some(name)) = (caps.get(1), caps.get(2)) {
                if let Ok(id) = id_str.as_str().parse::<i32>() {
                    map.insert(id, name.as_str().to_string());
                }
            }
        }
        map
    }

    fn get_proto_definitions() -> HashSet<String> {
        let mut protos = HashSet::new();
        if let Ok(entries) = fs::read_dir("src/proto") {
            let re_msg = Regex::new(r"message\s+(\w+)").unwrap();
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("proto") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            for caps in re_msg.captures_iter(&content) {
                                if let Some(name) = caps.get(1) {
                                    protos.insert(name.as_str().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        protos
    }

    fn normalize_proto_name(name: &str) -> String {
        name.to_lowercase()
            .replace("_", "")
    }

    fn derive_expected_name(pdf_name: &str) -> String {
        let mut s = pdf_name.trim().to_string();
        // check suffix case-insensitive
        if s.to_lowercase().ends_with(" request") {
            // Remove last 8 chars (" request")
            let base = &s[..s.len() - 8];
            s = format!("Request {}", base);
        } else if s.to_lowercase().ends_with(" response") {
            // Remove last 9 chars (" response")
            let base = &s[..s.len() - 9];
            s = format!("Response {}", base);
        }
        
        s.replace(" ", "")
         .replace("-", "")
         .replace("_", "")
         .to_lowercase()
    }

    #[test]
    fn verify_api_coverage() {
        let specs = extract_specs_from_pdf();
        if specs.is_empty() {
            println!("No API specs extracted from PDF. Skipping full compliance check.");
            return; 
        }

        let implemented_reqs = get_implemented_requests();
        let implemented_resps = get_implemented_responses();
        let proto_defs = get_proto_definitions();
        
        let mut implemented_all = implemented_reqs.clone();
        implemented_all.extend(implemented_resps.clone());

        let mut seen_ids = HashSet::new();
        let mut missing_count = 0;
        let mut failure_detected = false;
        
        let mut full_compliance_table = Vec::new();
        let mut matched_protos = HashSet::new();
        
        println!("\n{:-<5} | {:-<45} | {:-<10} | {:-<35} | {:-<10}", "", "", "", "", "");
        println!("{:^5} | {:<45} | {:<10} | {:<35} | {:^10}", "ID", "TEMPLATE NAME (PDF)", "DIR", "PROTO MESSAGE", "STATUS");
        println!("{:-<5} | {:-<45} | {:-<10} | {:-<35} | {:-<10}", "", "", "", "", "");

        for spec in &specs {
            seen_ids.insert(spec.id);
            
            let is_client = spec.direction.to_lowercase().contains("client");
            let impl_entry = if is_client {
                implemented_reqs.get(&spec.id)
            } else {
                implemented_resps.get(&spec.id)
            };

            // Determine Expected Proto Name from PDF
            let expected_name = derive_expected_name(&spec.name);
            
            // Try to find a matching proto definition
            let proto_match = if let Some(impl_name) = impl_entry {
                // If implemented, it should be in proto_defs
                if proto_defs.contains(impl_name) {
                    Some(impl_name.clone())
                } else {
                    None
                }
            } else {
                // If not implemented, search by expected name
                proto_defs.iter().find(|p| {
                    normalize_proto_name(p) == expected_name
                }).cloned()
            };

            if let Some(ref p) = proto_match {
                matched_protos.insert(p.clone());
            }

            let (status_icon, proto_display) = match impl_entry {
                Some(impl_name) => {
                     // Check if implemented struct exists in protos
                     let exact_match = proto_defs.contains(impl_name);
                     
                     if exact_match {
                          ("✅", impl_name.as_str())
                     } else {
                          // Fallback: Check normalized match (just in case case differs)
                          let n_impl = normalize_proto_name(impl_name);
                          if let Some(p_found) = proto_defs.iter().find(|p| normalize_proto_name(p) == n_impl) {
                              matched_protos.insert(p_found.clone());
                              ("✅", p_found.as_str())
                          } else {
                              failure_detected = true;
                              ("❌ NoProto", "-")
                          }
                     }
                },
                None => {
                    // Check if it exists in the OTHER map (wrong direction)
                    let wrong_map_entry = if is_client { implemented_resps.get(&spec.id) } else { implemented_reqs.get(&spec.id) };
                    
                    if let Some(name) = wrong_map_entry {
                        failure_detected = true;
                        ("❌ Dir", name.as_str())
                    } else {
                        // Completely missing implementation
                        // Check if we found a proto fuzzy match
                        if let Some(ref p_name) = proto_match {
                             missing_count += 1;
                             failure_detected = true;
                             ("❌ Logic", p_name.as_str()) // Proto exists but logic missing
                        } else {
                             missing_count += 1;
                             failure_detected = true;
                             ("❌ Missing", "-")
                        }
                    }
                }
            };

            // Truncate name if too long for table
            let name_display = if spec.name.len() > 45 {
                format!("{}...", &spec.name[..42])
            } else {
                spec.name.clone()
            };
            
            println!("{:^5} | {:<45} | {:<10} | {:<35} | {:^10}", 
                spec.id, name_display, spec.direction.replace("From ", ""), proto_display, status_icon);
            
            full_compliance_table.push((
                format!("{}", spec.id),
                spec.name.clone(),
                proto_display.to_string(),
                status_icon.to_string()
            ));
        }
        
        // Filter out generic/noise protos
        let ignored_protos = ["MessageType", "enum", "RequestOcoOrder", "RequestOCOOrder", "ResponseOCOOrder", "ResponseOcoOrder"];
        
        // Consolidate Extra and Undocumented into one list/table
        let mut additional_defs_for_print = Vec::new(); // For CLI output
        
        // Add Implemented Extras
        let mut extra_ids: Vec<i32> = implemented_all.keys().cloned().filter(|id| !seen_ids.contains(id)).collect();
        extra_ids.sort();
        
        for id in extra_ids {
            let name = implemented_all.get(&id).unwrap();
            matched_protos.insert(name.clone()); 
            additional_defs_for_print.push((format!("{}", id), name.clone(), "Implemented (Extra)"));
            
            full_compliance_table.push((
                format!("{}", id),
                "-".to_string(), // Template Name (PDF)
                name.clone(),    // Proto Message
                "⚠️ Extra".to_string()
            ));
        }

        // Add Proto Only (Undocumented)
        let mut undocumented_protos: Vec<String> = proto_defs.iter()
            .filter(|p| !matched_protos.contains(*p) && !ignored_protos.contains(&p.as_str()))
            .cloned()
            .collect();
        undocumented_protos.sort();

        for proto in undocumented_protos {
            additional_defs_for_print.push(("-".to_string(), proto.clone(), "Proto Only"));
            
            full_compliance_table.push((
                "-".to_string(), // ID
                "-".to_string(), // Template Name (PDF)
                proto,           // Proto Message
                "⚠️ Undocumented".to_string()
            ));
        }

        if !additional_defs_for_print.is_empty() {
             println!("\nAdditional Definitions (Not in PDF):");
             println!("{:-<10} | {:-<50} | {:-<20}", "", "", "");
             println!("{:^10} | {:<50} | {:^20}", "ID", "PROTO MESSAGE", "STATUS");
             println!("{:-<10} | {:-<50} | {:-<20}", "", "", "");
             
             for (id, name, status) in &additional_defs_for_print {
                 println!("{:^10} | {:<50} | {:^20}", id, name, status);
             }
             println!("{:-<10} | {:-<50} | {:-<20}", "", "", "");
        }
        
        // Update README.md with FULL table
        update_readme_with_compliance(&full_compliance_table);

        println!("\nCoverage Summary:");
        println!("Total Templates (PDF): {}", specs.len());
        println!("Implemented (Matched): {}", specs.len() - missing_count);
        println!("Missing:               {}", missing_count);

        if failure_detected {
            panic!("Compliance check failed: mismatched names, missing definitions, or extra implementations detected.");
        }
    }

    fn update_readme_with_compliance(defs: &[(String, String, String, String)]) {
        let readme_path = Path::new("README.md");
        if let Ok(content) = fs::read_to_string(readme_path) {
            let header = "## API Compliance Status";
            let mut new_content = String::new();
            
            // Cut off existing section if it exists
            if let Some(idx) = content.find(header) {
                new_content.push_str(&content[..idx]);
            } else {
                new_content.push_str(&content);
                if !new_content.ends_with('\n') { new_content.push('\n'); }
                new_content.push('\n');
            }

            new_content.push_str(header);
            new_content.push_str("\n\n| ID | Template Name | Proto Message | Status |\n|---|---|---|---|\n");
            for (id, template, proto, status) in defs {
                new_content.push_str(&format!("| {} | {} | {} | {} |\n", id, template, proto, status));
            }
            
            let _ = fs::write(readme_path, new_content);
        }
    }
}
