import datetime
import os
import sys
import json
import pandas as pd
import shutil
import argparse
import openpyxl

def update_excel_with_tactical_output(json_data, excel_file, work_order_col, activity_col):
    # Load the Excel file
    df = pd.read_excel(excel_file)
  

    # Load the Excel file into a DataFrame

    #df.insert(0, 'Start Day', 0)
    df = df.dropna(subset=[work_order_col])
    df[work_order_col] = df[work_order_col].astype(int)
    df[activity_col] = df[activity_col].astype(int)

    print(f"{df[df['Order'] == 2100049119]}")
    # Iterate over each item in JSON data
    for work_order_number, work_order in json_data.items():
        # Find rows where search_col matches id_key
  
        for activity, start_day in work_order.items():
            # Check if update_col contains update_key
            # Update the corresponding cell with update_value
            date_string = start_day
            date_string = date_string.replace("Z", "")
            date_object = datetime.datetime.fromisoformat(date_string).date()
            # print(f"Start day {date_object}")
            # print(f"Work order {df[work_order_col]}")
            # print(f"Activity {df[activity_col]}")
            # print(f"dataframe {df[work_order_col]}")
            # print(f"dataframe {df[activity_col]}")
            # if not df[work_order_col].__contains__(int(work_order_number)):
                # print(f"work order exist {df[work_order_col] == int(work_order_number)}")
                # print(f"activity exist {df[activity_col] == int(activity)}")
            df.loc[(df[work_order_col] == int(work_order_number)) & (df[activity_col] == int(activity)), 'Start Day'] = str(date_object)

    # Save the updated DataFrame back to the Excel file
    df.to_excel(excel_file, index=False)

def main():
    # print working directory 
    print('Current working directory:', os.getcwd())

    output_path = 'output/output-for-valentin-sap-updater-2024-03-13.xlsx'

    # # Check if the file exists to avoid an error
    # if os.path.exists(output_path):
    #     os.remove(output_path)
    #     print(f"File {output_path} has been deleted.")
    # else:
    #     print(f"The file {output_path} does not exist.")
    # shutil.copy('scheduling_system/test_data/valentin-sap-updater-2024-03-13.xlsx', output_path)


    # Read JSON data from stdin
    json_str = sys.stdin.read()
    json_data = json.loads(json_str)


    # Define the Excel file, search column, and update column
    excel_file = './output/output-for-valentin-sap-updater-2024-03-13-corrected.xlsx'
    work_order_col = 'Order'  # Replace with your actual column name
    activity_col = 'Activity' # Replace with your actual column name

    # Update the Excel file based on JSON data
    update_excel_with_tactical_output(json_data['tactical_agent_solution'], excel_file, work_order_col, activity_col)

if __name__ == "__main__":
    main()