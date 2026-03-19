#!/usr/bin/env python3
import sys, json, re, argparse

try:
    import spacy
    try:
        nlp = spacy.load("en_core_web_lg")
    except OSError:
        try:
            nlp = spacy.load("en_core_web_md")
        except OSError:
            nlp = spacy.load("en_core_web_sm")
except ImportError:
    print(json.dumps([]))
    sys.exit(0)

SKIP_LABELS = {"CARDINAL", "ORDINAL", "QUANTITY", "PERCENT", "MONEY", "TIME"}

INDIAN_NAMES = {
    "arjun","priya","rahul","ananya","vikram","meera","kiran","divya",
    "siddharth","kavya","rohan","aisha","ashvanthan","adhithya","abishek",
    "suresh","ramesh","ganesh","mahesh","rajesh","dinesh","mukesh",
    "lakshmi","saraswathi","parvathi","durga","devi","geetha","radha",
    "kumar","shankar","krishna","venkat","narayana","subramanian","murugan",
    "selvam","rajan","mohan","gopal","balaji","senthil","kartik","surya",
    "dharani","harini","nithya","keerthana","sowmya","lavanya","deepika",
    "arun","varun","tarun","nitin","vivek","satish","manish","girish",
    "pooja","neha","sneha","rekha","usha","asha","latha","vijaya",
    "ashwin","ashwini","krishnamurthy","subramaniam","balamurugan","saravanan",
    "manikandan","vignesh","pradeep","prakash","praveen","premkumar",
    "raghu","ravi","sanjay","sathish","shiva","shravan","thilak",
    "udhay","vijay","vinod","yogesh","yuvaraj","adhi","adhith"
}

AGE_RE = re.compile(
    r'\b(?:age[sd]?\s+(\d{1,3})|(\d{1,3})\s*[-\s]?year[s]?\s*[-\s]?old|(\d{1,3})\s+yrs?\.?(?:\s+old)?)\b',
    re.IGNORECASE
)

DOB_PHRASE_RE = re.compile(
    r'\b(?:born\s+on|D\.?O\.?B\.?|date\s+of\s+birth)\s*:?\s*'
    r'(\d{1,2}[\/\-\.]\d{1,2}[\/\-\.]\d{2,4}'
    r'|(?:jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)\w*\s+\d{1,2},?\s+\d{4}'
    r'|\d{4}[\/\-\.]\d{1,2}[\/\-\.]\d{1,2})',
    re.IGNORECASE
)

NAMED_RE = re.compile(
    r'\b(?:named|name\s+is|called|known\s+as)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)',
    re.IGNORECASE
)

# Department patterns — match full phrase including "dept" / "department" suffix
DEPT_RE = re.compile(
    r'\b('
    r'(?:computer\s*science|cse|c\.s\.e)'
    r'|(?:information\s*technology|it\s*dept|i\.t\.)'
    r'|(?:electrical\s*(?:and\s*)?electronics|eee|e\.e\.e|ece|e\.c\.e)'
    r'|(?:mechanical(?:\s*engg?)?)'
    r'|(?:civil(?:\s*engg?)?)'
    r'|(?:chemical(?:\s*engg?)?)'
    r'|(?:biomedical|bio-medical)'
    r'|(?:aerospace|aeronautical)'
    r'|(?:artificial\s*intelligence|ai\s*(?:and|&)\s*ds|ai\s*dept)'
    r'|(?:data\s*science)'
    r'|(?:mathematics?(?:\s*dept)?)'
    r'|(?:management|mba)'
    r')\s*(?:dept(?:artment)?\.?)?',
    re.IGNORECASE
)

def overlaps(s1, e1, spans):
    return any(not (e1 <= s or e <= s1) for s, e in spans)

def is_likely_person(text):
    words = text.lower().split()
    if any(w in INDIAN_NAMES for w in words):
        return True
    if re.match(r'^(mr|mrs|ms|dr|prof|shri|smt)\.?\s', text, re.IGNORECASE):
        return True
    return False

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--text", required=True)
    args = parser.parse_args()
    text = args.text

    entities = []
    seen = []

    # Step 1: AGE
    for m in AGE_RE.finditer(text):
        s, e = m.start(), m.end()
        if not overlaps(s, e, seen):
            seen.append((s, e))
            entities.append({"original": m.group(), "entity_type": "AGE_NUM", "start": s, "end": e})

    # Step 2: DOB — only the date portion
    for m in DOB_PHRASE_RE.finditer(text):
        date_str = m.group(1)
        s, e = m.start(1), m.end(1)
        if not overlaps(s, e, seen):
            seen.append((s, e))
            entities.append({"original": date_str, "entity_type": "DOB", "start": s, "end": e})

    # Step 3: DEPARTMENT — full phrase including "dept" word
    for m in DEPT_RE.finditer(text):
        s, e = m.start(), m.end()
        # trim trailing space
        matched = m.group().rstrip()
        e = s + len(matched)
        if not overlaps(s, e, seen):
            seen.append((s, e))
            entities.append({"original": matched, "entity_type": "ORG", "start": s, "end": e})

    # Step 4: spaCy NER
    doc = nlp(text)
    for ent in doc.ents:
        if ent.label_ in SKIP_LABELS:
            continue
        if overlaps(ent.start_char, ent.end_char, seen):
            continue
        label = ent.label_
        if label in ("GPE", "LOC") and is_likely_person(ent.text):
            label = "PERSON"
        if label == "DATE":
            if re.search(r'\b\d{1,3}\s*(year|yr|age)', ent.text, re.IGNORECASE):
                continue
        seen.append((ent.start_char, ent.end_char))
        entities.append({"original": ent.text, "entity_type": label, "start": ent.start_char, "end": ent.end_char})

    # Step 5: "named X" pattern
    for m in NAMED_RE.finditer(text):
        name = m.group(1)
        s, e = m.start(1), m.end(1)
        if not overlaps(s, e, seen):
            seen.append((s, e))
            entities.append({"original": name, "entity_type": "PERSON", "start": s, "end": e})

    entities.sort(key=lambda x: x["start"])
    print(json.dumps(entities))

if __name__ == "__main__":
    main()
