---
alwaysApply: true
---

# Commandments always to be followed with each prompt or iteration

1. When using MCP Tools to make changes to the project, always adhere to these commandments.

2. ALWAYS use directory_tree, search_files, list_directory and get a detailed understanding of all relevant files and directories before attempting to write_file at path. Avoid assumptions, verify and know the project's actual contents.

3. NEVER attempt to use write_file or edit_file without first verifying the destination path exists. When it is necessary to create a new directory, use create_directory. This MUST be done before creating a file at destination path.

4. MCP Tools allows line edits with edit_file. Whenever practical, make line-based edits. Each edit replaces exact line sequences with new content. Returns a git-style diff showing the changes made. When editing a file, make sure the code is still complete. NEVER use placeholders.

5. ALWAYS check and verify if a file already exists before attempting to write or edit a file. If file exists, use read_file to read the complete contents of a file. For files that include "import" or otherwise reference other files, use read_multiple_files to read the contents of multiple files simultaneously. This is more efficient than reading files one by one when you need to analyze or compare multiple files.

6. If write_file is being used, the entire file's contents must be written. ALWAYS write complete code and NEVER use placeholders.  

7. When updating CHANGELOG.md always use edit_file.

8. When updating other documentation (e.g., README.md) always use edit_file.

9. When important decisions about architecture, design, dependencies, or frameworks need to be made, discuss options with me first. Weight the pros and cons and then tell me which option you believe is best and the reason why. Then ask for my final decision or new input.

10. If and when command lines need to be entered into VS Code terminal, please provide the full path as well as the exact commands to execute. Wait for me to share back the response before proceeding.

11. Create .md files to document all your changes and decisions and always put them into "changelogs/"