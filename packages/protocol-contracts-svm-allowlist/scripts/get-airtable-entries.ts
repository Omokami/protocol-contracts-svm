import { fetchAllAirtableRecords } from "../src/get-airtable-entries";
import { writeRawToCSV } from "../src/utils";

const main = async () => {
  const rawEntries = await fetchAllAirtableRecords();
  await writeRawToCSV("raw-burners.csv", rawEntries);
};

main();
