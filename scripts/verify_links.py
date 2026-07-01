import os
import re
import urllib.parse
import sys

# Regex for inline markdown links: [text](link)
INLINE_LINK_RE = re.compile(r'\[([^\]]*?)\]\(([^)]+?)\)')

# Regex for reference-style link definitions: [ref]: link
REF_LINK_RE = re.compile(r'^\[([^\]]+)\]:\s*([^\s]+)', re.MULTILINE)

def find_markdown_files(root_dir):
    md_files = []
    ignored_dirs = {'.git', 'target', 'node_modules', '.agents', '.lh', '.config', 'registry'}
    for root, dirs, files in os.walk(root_dir):
        dirs[:] = [d for d in dirs if d not in ignored_dirs]
        for file in files:
            if file.endswith('.md'):
                md_files.append(os.path.join(root, file))
    return md_files

def check_links():
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
    md_files = find_markdown_files(root_dir)
    
    truly_broken_count = 0
    non_portable_existing_count = 0
    non_portable_broken_count = 0
    checked_links_count = 0
    
    print(f"Starting link verification for {len(md_files)} markdown files...")
    
    for file_path in sorted(md_files):
        rel_file_path = os.path.relpath(file_path, root_dir)
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
            
        inline_matches = INLINE_LINK_RE.findall(content)
        ref_matches = REF_LINK_RE.findall(content)
        
        links = []
        for text, url in inline_matches:
            links.append((url, "inline"))
        for ref, url in ref_matches:
            links.append((url, f"reference [{ref}]"))
            
        file_dir = os.path.dirname(file_path)
        
        for url, link_type in links:
            url = url.strip()
            if url.startswith('<') and url.endswith('>'):
                url = url[1:-1].strip()
                
            if not url or url.startswith('#') or url.startswith('http://') or url.startswith('https://') or url.startswith('mailto:') or url.startswith('ftp://'):
                continue
                
            checked_links_count += 1
            
            decoded_url = urllib.parse.unquote(url)
            
            # Check for non-portable file:/// links
            if decoded_url.startswith('file://'):
                path_part = decoded_url[7:].split('?')[0].split('#')[0]
                target_path = path_part
                
                # Check if it actually exists on this system
                exists = os.path.exists(target_path)
                if exists:
                    print(f"⚠️  Non-portable absolute file:// link (EXISTS on disk) in {rel_file_path}:")
                    print(f"   Link: '{url}'")
                    non_portable_existing_count += 1
                else:
                    print(f"❌ Broken non-portable file:// link (DOES NOT EXIST) in {rel_file_path}:")
                    print(f"   Link: '{url}'")
                    non_portable_broken_count += 1
                continue
            
            path_part = decoded_url.split('?')[0].split('#')[0]
            
            if not path_part:
                continue
                
            if path_part.startswith('/'):
                target_path = os.path.join(root_dir, path_part.lstrip('/'))
            else:
                target_path = os.path.join(file_dir, path_part)
                
            target_path = os.path.abspath(target_path)
            
            if not os.path.exists(target_path):
                print(f"❌ Broken {link_type} link in {rel_file_path}:")
                print(f"   Link: '{url}'")
                print(f"   Resolved to: '{os.path.relpath(target_path, root_dir)}'")
                truly_broken_count += 1
                
    print(f"\nSummary:")
    print(f"Checked links: {checked_links_count}")
    print(f"Non-portable file:// links (existing): {non_portable_existing_count}")
    print(f"Non-portable file:// links (broken): {non_portable_broken_count}")
    print(f"Truly broken relative/local links: {truly_broken_count}")
    
    total_broken = truly_broken_count + non_portable_broken_count + non_portable_existing_count
    if total_broken > 0:
        sys.exit(1)
    else:
        print("✅ All local/relative markdown links exist and are portable!")
        sys.exit(0)

if __name__ == '__main__':
    check_links()
