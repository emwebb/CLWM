CREATE TABLE "change_set" (
	"change_set_id" INTEGER NOT NULL UNIQUE,
	"change_date" INTEGER NOT NULL,
	"change_source" TEXT NOT NULL,
	PRIMARY KEY("change_set_id" AUTOINCREMENT)
);
CREATE TABLE "data_type" (
	"data_type_name" TEXT NOT NULL,
	"system_defined" INTEGER NOT NULL,
	"definition" BLOB NOT NULL,
	"version" INTEGER NOT NULL,
	"change_set_id" INTEGER NOT NULL,
	PRIMARY KEY("version", "data_type_name"),
	FOREIGN KEY("change_set_id") REFERENCES "change_set"("change_set_id")
);
CREATE TABLE "noun" (
	"noun_id" INTEGER NOT NULL UNIQUE,
	"name" TEXT NOT NULL,
	"last_change_set_id" INTEGER NOT NULL,
	"noun_type_id" INTEGER NOT NULL,
	"metadata" TEXT NOT NULL,
	PRIMARY KEY("noun_id" AUTOINCREMENT),
	FOREIGN KEY("noun_type_id") REFERENCES "noun_type"("noun_type_id"),
	FOREIGN KEY("last_change_set_id") REFERENCES "change_set"("change_set_id")
);
CREATE TABLE "noun_history" (
	"noun_id" INTEGER NOT NULL,
	"change_set_id" INTEGER NOT NULL,
	"diff_name" TEXT NOT NULL,
	"diff_noun_type" TEXT NOT NULL,
	"diff_metadata" TEXT NOT NULL,
	FOREIGN KEY("change_set_id") REFERENCES "change_set"("change_set_id"),
	FOREIGN KEY("noun_id") REFERENCES "noun"("noun_id"),
	PRIMARY KEY("noun_id", "change_set_id")
);
CREATE TABLE "noun_type" (
	"noun_type_id" INTEGER NOT NULL UNIQUE,
	"last_change_set_id" INTEGER NOT NULL,
	"noun_type" TEXT NOT NULL UNIQUE,
	"metadata" TEXT NOT NULL,
	FOREIGN KEY("last_change_set_id") REFERENCES "change_set"("change_set_id"),
	PRIMARY KEY("noun_type_id" AUTOINCREMENT)
);
CREATE TABLE "noun_type_history" (
	"noun_type_id" INTEGER NOT NULL,
	"change_set_id" INTEGER NOT NULL,
	"diff_noun_type" TEXT NOT NULL,
	"diff_metadata" TEXT NOT NULL,
	FOREIGN KEY("noun_type_id") REFERENCES "noun_type"("noun_type_id"),
	FOREIGN KEY("change_set_id") REFERENCES "change_set"("change_set_id"),
	PRIMARY KEY("change_set_id", "noun_type_id")
);
CREATE TABLE "attribute_type" (
	"attribute_type_id" INTEGER NOT NULL,
	"attribute_name" TEXT NOT NULL,
	"data_type_name" TEXT NOT NULL,
	"multiple_allowed" INTEGER NOT NULL,
	"metadata" TEXT NOT NULL,
	"last_change_set_id" INTEGER NOT NULL,
	PRIMARY KEY("attribute_type_id" AUTOINCREMENT),
	FOREIGN KEY("data_type_name") REFERENCES "data_type"("data_type_name"),
	FOREIGN KEY("last_change_set_id") REFERENCES "change_set"("change_set_id")
);
CREATE TABLE "attribute_type_history" (
	"attribute_type_id" INTEGER NOT NULL,
	"change_set_id" INTEGER NOT NULL,
	"diff_attribute_name" TEXT NOT NULL,
	"diff_multiple_allowed" TEXT NOT NULL,
	"diff_metadata" TEXT NOT NULL,
	FOREIGN KEY("attribute_type_id") REFERENCES "attribute_type"("attribute_type_id"),
	FOREIGN KEY("change_set_id") REFERENCES "change_set"("change_set_id"),
	PRIMARY KEY("change_set_id", "attribute_type_id")
);