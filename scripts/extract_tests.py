import os
import re

def write_test_functions(test_functions):
    with open('../tests.rs', 'w') as test_file:
        counter = 0
        for test_function in test_functions:
            test_file.write(f"{test_function}")
            if counter < len(test_functions) - 1:
                test_file.write("\n\n")
            counter += 1
    print(f"Wrote {counter} test functions to 'tests.rs'.")

def build_function(directory, file_name, has_error = False):
    function_string = "    #[test]\n"
    function_string += f"    fn {directory}_{file_name.replace('.txt', '')}() {{\n"
    if has_error:
        function_string += "        let result = std::panic::catch_unwind(|| {\n"
        function_string += f"            run_test(\"{directory}\", \"{file_name.replace('.txt', '')}\")\n"
        function_string += "        });\n"
        function_string += "        assert!(result.is_err(), \"Expected a panic but did not get one\");\n"
    else:
        function_string += f"        match run_test(\"{directory}\", \"{file_name.replace('.txt', '')}\") {{\n"
        function_string += "            Ok(_) => assert!(true),\n"
        function_string += "            Err(err) => assert!(false, \"{}\", err),\n"
        function_string += "        }\n"
    function_string += "    }"
    return function_string

def parse_file(input_file):
    expect_comments = []
    has_error = False
                    
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
                is_literal = expect_value in ['true', 'false', 'nil']  # Check if the value is a literal
                if not is_number and not is_literal:
                    expect_value = f'"{expect_value}"'
                expect_comments.append(expect_value)
            elif "error" in comment.lower():
                has_error = True
    
    return expect_comments, has_error

def main(input_dir, output_dir, test_dir):
    # Create output directory if it doesn't exist
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # Create test output directory if it doesn't exist
    if not os.path.exists(test_dir):
        os.makedirs(test_dir)
    
    # Define the rust test functions
    test_functions = []

    # Loop through all subdirectories in the input directory
    for directory in os.listdir(input_dir):
        # Check if directory is actually a directory
        if not os.path.isdir(os.path.join(input_dir, directory)):
            continue

        # Define the input and output subdirectory paths
        input_directory_path = os.path.join(input_dir, directory)
        output_directory_path = os.path.join(output_dir, directory)

        # Create output subdirectory if it doesn't exist
        if not os.path.exists(output_directory_path):
            os.makedirs(output_directory_path)
        
        # Create test output subdirectory if it doesn't exist
        test_output_directory_path = os.path.join(test_dir, directory)
        if not os.path.exists(test_output_directory_path):
            os.makedirs(test_output_directory_path)

        # Loop through all files in the input subdirectory
        for file_name in os.listdir(input_directory_path):
            input_file_path = os.path.join(input_directory_path, file_name)
            
            # Make sure it's a file
            if os.path.isfile(input_file_path):
                with open(input_file_path, 'r') as input_file:
                    expect_comments, has_error = parse_file(input_file)
                
                # Define the corresponding output file path
                file_name = file_name.replace(".lox", ".txt")
                output_file_path = os.path.join(output_directory_path, f"{file_name}")
                
                # Write the extracted "expect:" comments to the new file
                with open(output_file_path, 'w') as output_file:
                    for comment in expect_comments:
                        output_file.write(comment + '\n')
                
                # Build the test function string
                test_function = build_function(directory, file_name, has_error)
                test_functions.append(test_function)
                
                print(f"Extracted {len(expect_comments)} 'expect:' comments from '{input_file_path}' and saved in '{output_file_path}'.")

    print(f"Expect comments extraction completed. Output saved in '{output_dir}'.")

    # Write the test functions to a file
    write_test_functions(test_functions)

# Example usage
input_directory = '../tests'   # Path to the directory containing the input files
output_directory = '../output/expected' # Path to the directory where the expected output files will be saved
test_output_directory = '../output/actual' # Path to the directory where the actual output files will be saved

main(input_directory, output_directory, test_output_directory)
