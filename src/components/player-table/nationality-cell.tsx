import zanzibarFlag from "@/assets/flags/zanzibar.svg";

type NationalityFlag =
  | { type: "package"; countryCode: string }
  | { type: "asset"; source: string };

const countryCodes: Record<string, string> = {
  Afghanistan: "AF",
  Albania: "AL",
  Algeria: "DZ",
  "American Samoa": "AS",
  Andorra: "AD",
  Angola: "AO",
  Anguilla: "AI",
  "Antigua & Barbuda": "AG",
  Argentina: "AR",
  Armenia: "AM",
  Aruba: "AW",
  Australia: "AU",
  Austria: "AT",
  Azerbaijan: "AZ",
  Bahamas: "BS",
  Bahrain: "BH",
  Bangladesh: "BD",
  Barbados: "BB",
  Belarus: "BY",
  Belgium: "BE",
  Belize: "BZ",
  Benin: "BJ",
  Bermuda: "BM",
  Bhutan: "BT",
  Bolivia: "BO",
  Bonaire: "BQ-BO",
  "Bosnia & Herzegovina": "BA",
  Botswana: "BW",
  Brazil: "BR",
  "British Virgin Islands": "VG",
  Brunei: "BN",
  Bulgaria: "BG",
  "Burkina Faso": "BF",
  Burundi: "BI",
  Cambodia: "KH",
  Cameroon: "CM",
  Canada: "CA",
  "Cape Verde": "CV",
  "Cayman Islands": "KY",
  "Central African Republic": "CF",
  Chad: "TD",
  Chile: "CL",
  "China PR": "CN",
  "Chinese Taipei": "TW",
  Colombia: "CO",
  Comoros: "KM",
  Congo: "CG",
  "Cook Islands": "CK",
  "Costa Rica": "CR",
  Croatia: "HR",
  Cuba: "CU",
  Curaçao: "CW",
  Cyprus: "CY",
  Czechia: "CZ",
  "DR Congo": "CD",
  Denmark: "DK",
  Djibouti: "DJ",
  Dominica: "DM",
  "Dominican Republic": "DO",
  Ecuador: "EC",
  Egypt: "EG",
  "El Salvador": "SV",
  England: "GB-ENG",
  "Equatorial Guinea": "GQ",
  Eritrea: "ER",
  Estonia: "EE",
  Eswatini: "SZ",
  Ethiopia: "ET",
  "Faroe Islands": "FO",
  Fiji: "FJ",
  Finland: "FI",
  France: "FR",
  "French Guiana": "GF",
  Gabon: "GA",
  Georgia: "GE",
  Germany: "DE",
  Ghana: "GH",
  Gibraltar: "GI",
  Greece: "GR",
  Grenada: "GD",
  Guadeloupe: "GP",
  Guam: "GU",
  Guatemala: "GT",
  Guinea: "GN",
  "Guinea-Bissau": "GW",
  Guyana: "GY",
  Haiti: "HT",
  Honduras: "HN",
  "Hong Kong": "HK",
  Hungary: "HU",
  Iceland: "IS",
  India: "IN",
  Indonesia: "ID",
  Iran: "IR",
  Iraq: "IQ",
  Ireland: "IE",
  Israel: "IL",
  Italy: "IT",
  "Ivory Coast": "CI",
  Jamaica: "JM",
  Japan: "JP",
  Jordan: "JO",
  Kazakhstan: "KZ",
  Kenya: "KE",
  Kosovo: "XK",
  Kuwait: "KW",
  Kyrgyzstan: "KG",
  Laos: "LA",
  Latvia: "LV",
  Lebanon: "LB",
  Lesotho: "LS",
  Liberia: "LR",
  Libya: "LY",
  Liechtenstein: "LI",
  Lithuania: "LT",
  Luxembourg: "LU",
  Macau: "MO",
  Madagascar: "MG",
  Malawi: "MW",
  Malaysia: "MY",
  Maldives: "MV",
  Mali: "ML",
  Malta: "MT",
  Martinique: "MQ",
  Mauritania: "MR",
  Mauritius: "MU",
  Mayotte: "YT",
  Mexico: "MX",
  Micronesia: "FM",
  Moldova: "MD",
  Mongolia: "MN",
  Montenegro: "ME",
  Montserrat: "MS",
  Morocco: "MA",
  Mozambique: "MZ",
  Myanmar: "MM",
  Namibia: "NA",
  Nepal: "NP",
  Netherlands: "NL",
  "New Caledonia": "NC",
  "New Zealand": "NZ",
  Nicaragua: "NI",
  Niger: "NE",
  Nigeria: "NG",
  "North Korea": "KP",
  "North Macedonia": "MK",
  "Northern Ireland": "GB-NIR",
  "Northern Mariana": "MP",
  Norway: "NO",
  Oman: "OM",
  Pakistan: "PK",
  Palestine: "PS",
  Panama: "PA",
  "Papua New Guinea": "PG",
  Paraguay: "PY",
  Peru: "PE",
  Philippines: "PH",
  Poland: "PL",
  Portugal: "PT",
  "Puerto Rico": "PR",
  Qatar: "QA",
  Romania: "RO",
  Russia: "RU",
  Rwanda: "RW",
  Réunion: "RE",
  "Saint Kitts & Nevis": "KN",
  "Saint Lucia": "LC",
  "Saint Vincent & the Grenadines": "VC",
  "Saint-Martin": "MF",
  Samoa: "WS",
  "San Marino": "SM",
  "Saudi Arabia": "SA",
  Scotland: "GB-SCT",
  Senegal: "SN",
  Serbia: "RS",
  Seychelles: "SC",
  "Sierra Leone": "SL",
  Singapore: "SG",
  "Sint Maarten": "SX",
  Slovakia: "SK",
  Slovenia: "SI",
  "Solomon Islands": "SB",
  Somalia: "SO",
  "South Africa": "ZA",
  "South Korea": "KR",
  "South Sudan": "SS",
  Spain: "ES",
  "Sri Lanka": "LK",
  "St. Barthélemy": "BL",
  Sudan: "SD",
  Suriname: "SR",
  Sweden: "SE",
  Switzerland: "CH",
  Syria: "SY",
  "São Tomé & Príncipe": "ST",
  Tahiti: "PF",
  Tajikistan: "TJ",
  Tanzania: "TZ",
  Thailand: "TH",
  "The Gambia": "GM",
  "Timor-Leste": "TL",
  Togo: "TG",
  Tonga: "TO",
  "Trinidad & Tobago": "TT",
  Tunisia: "TN",
  Turkmenistan: "TM",
  "Turks & Caicos Islands": "TC",
  Tuvalu: "TV",
  Türkiye: "TR",
  UAE: "AE",
  "US Virgin Islands": "VI",
  Uganda: "UG",
  Ukraine: "UA",
  "United States": "US",
  Uruguay: "UY",
  Uzbekistan: "UZ",
  Vanuatu: "VU",
  Venezuela: "VE",
  Vietnam: "VN",
  Wales: "GB-WLS",
  Yemen: "YE",
  Zambia: "ZM",
  Zimbabwe: "ZW",
};

export function nationalityFlagFor(
  nationality: string,
): NationalityFlag | undefined {
  if (nationality === "Zanzibar") {
    return { type: "asset", source: zanzibarFlag };
  }
  const countryCode = countryCodes[nationality];
  return countryCode ? { type: "package", countryCode } : undefined;
}

export function NationalityCell({
  nationalities,
}: {
  nationalities: readonly string[];
}) {
  if (nationalities.length === 0) {
    return "—";
  }

  return (
    <span className="flex h-full min-w-0 items-center gap-1">
      {nationalities.map((nationality, index) => {
        const key = `${nationality}-${index}`;
        const flag = nationalityFlagFor(nationality);
        if (!flag) {
          return (
            <span key={key} className="min-w-0 truncate" title={nationality}>
              {nationality}
            </span>
          );
        }
        if (flag.type === "asset") {
          return (
            <img
              key={key}
              alt={nationality}
              aria-label={nationality}
              className="block h-3.5 w-auto shrink-0"
              src={flag.source}
              title={nationality}
            />
          );
        }
        return (
          <span
            key={key}
            aria-label={nationality}
            className={`flag:${flag.countryCode} block shrink-0 [--CountryFlagIcon-height:0.875rem]`}
            role="img"
            title={nationality}
          />
        );
      })}
    </span>
  );
}
