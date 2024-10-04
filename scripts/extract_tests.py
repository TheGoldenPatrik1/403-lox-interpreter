import os
import re

def extract_expect_comments(input_dir, output_dir):
    # Create output directory if it doesn't exist
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
    
    # Loop through all files in the input directory
    for file_name in os.listdir(input_dir):
        input_file_path = os.path.join(input_dir, file_name)
        
        # Make sure it's a file
        if os.path.isfile(input_file_path):
            with open(input_file_path, 'r') as input_file:
                expect_comments = []
                
                # Read through each line of the file
                for line in input_file:
                    line = line.strip()
                    
                    # Check if the line contains a comment ("//")
                    comment_index = line.find("//")
                    if comment_index != -1:
                        comment = line[comment_index + 2:].strip()  # Extract the comment part
                        
                        # Check if the comment contains "expect:"
                        if "expect:" in comment:
                            expect_value = comment.split("expect:")[1].strip()  # Extract the value after "expect:"
                            is_number = re.match(r'^-?\d+(?:\.\d+)?$', expect_value) is not None  # Check if the value is a number
                            if not is_number:
                                expect_value = f'"{expect_value}"'
                            expect_comments.append(expect_value)
            
            # Define the corresponding output file path
            file_name = file_name.replace(".lox", ".txt")
            output_file_path = os.path.join(output_dir, f"{file_name}")
            
            # Write the extracted "expect:" comments to the new file
            with open(output_file_path, 'w') as output_file:
                for comment in expect_comments:
                    output_file.write(comment + '\n')
            
            print(f"Extracted {len(expect_comments)} 'expect:' comments from '{input_file_path}' and saved in '{output_file_path}'.")

    print(f"Expect comments extraction completed. Output saved in '{output_dir}'.")

# Example usage
input_directory = '../tests'   # Path to the directory containing the input files
output_directory = '../output/expected' # Path to the directory where the comment files will be saved

extract_expect_comments(input_directory, output_directory)
