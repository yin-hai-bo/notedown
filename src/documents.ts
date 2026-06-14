import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, reactive } from "vue";

type MenuActionId =
  | "file.new"
  | "file.open"
  | "file.save"
  | "file.save_as"
  | "file.preferences";

interface MenuActionPayload {
  id: MenuActionId;
}

interface DocumentData {
  path: string;
  title: string;
  content: string;
}

export interface DocumentSession {
  id: string;
  title: string;
  path: string | null;
  content: string;
  savedContent: string;
}

interface DocumentState {
  documents: DocumentSession[];
  activeDocumentId: string | null;
  nextUntitledIndex: number;
}

const MENU_ACTION_EVENT = "menu-action";

export const documentState = reactive<DocumentState>({
  documents: [],
  activeDocumentId: null,
  nextUntitledIndex: 1,
});

export const activeDocument = computed(() => {
  return documentState.documents.find((document) => document.id === documentState.activeDocumentId) ?? null;
});

export function isDirty(document: DocumentSession): boolean {
  return document.content !== document.savedContent;
}

function createDocumentId(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

function titleFromPath(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  return normalized.split("/").pop() || "未命名";
}

function activateDocument(id: string): void {
  documentState.activeDocumentId = id;
}

function upsertOpenedDocument(data: DocumentData): void {
  const existing = documentState.documents.find((document) => document.path === data.path);

  if (existing) {
    existing.title = data.title || titleFromPath(data.path);
    existing.content = data.content;
    existing.savedContent = data.content;
    activateDocument(existing.id);
    return;
  }

  const document: DocumentSession = {
    id: createDocumentId(),
    title: data.title || titleFromPath(data.path),
    path: data.path,
    content: data.content,
    savedContent: data.content,
  };

  documentState.documents.push(document);
  activateDocument(document.id);
}

function applySavedDocument(document: DocumentSession, data: DocumentData): void {
  document.path = data.path;
  document.title = data.title || titleFromPath(data.path);
  document.content = data.content;
  document.savedContent = data.content;
}

export function createUntitledDocument(): DocumentSession {
  const title = `未命名-${documentState.nextUntitledIndex++}`;
  const document: DocumentSession = {
    id: createDocumentId(),
    title,
    path: null,
    content: "",
    savedContent: "",
  };

  documentState.documents.push(document);
  activateDocument(document.id);
  return document;
}

export function setActiveDocument(id: string): void {
  if (documentState.documents.some((document) => document.id === id)) {
    activateDocument(id);
  }
}

export async function openDocument(): Promise<void> {
  const opened = await invoke<DocumentData | null>("open_document");
  if (opened) {
    upsertOpenedDocument(opened);
  }
}

export async function saveDocument(document: DocumentSession): Promise<boolean> {
  if (!document.path) {
    return saveDocumentAs(document);
  }

  const saved = await invoke<DocumentData>("save_document", {
    path: document.path,
    content: document.content,
  });
  applySavedDocument(document, saved);
  return true;
}

export async function saveDocumentAs(document: DocumentSession): Promise<boolean> {
  const saved = await invoke<DocumentData | null>("save_document_as", {
    content: document.content,
  });

  if (!saved) {
    return false;
  }

  applySavedDocument(document, saved);
  return true;
}

export async function closeDocument(document: DocumentSession): Promise<void> {
  if (isDirty(document)) {
    const shouldClose = window.confirm(`"${document.title}" 有未保存修改。关闭前请先保存，是否继续关闭？`);
    if (!shouldClose) {
      return;
    }
  }

  const index = documentState.documents.findIndex((item) => item.id === document.id);
  if (index < 0) {
    return;
  }

  documentState.documents.splice(index, 1);

  if (documentState.activeDocumentId !== document.id) {
    return;
  }

  const nextDocument = documentState.documents[index] ?? documentState.documents[index - 1] ?? null;
  documentState.activeDocumentId = nextDocument?.id ?? null;
}

async function saveActiveDocument(): Promise<void> {
  const document = activeDocument.value;
  if (document) {
    await saveDocument(document);
  }
}

async function saveActiveDocumentAs(): Promise<void> {
  const document = activeDocument.value;
  if (document) {
    await saveDocumentAs(document);
  }
}

export async function watchMenuActions(): Promise<() => void> {
  return listen<MenuActionPayload>(MENU_ACTION_EVENT, async (event) => {
    switch (event.payload.id) {
      case "file.new":
        createUntitledDocument();
        break;
      case "file.open":
        await openDocument();
        break;
      case "file.save":
        await saveActiveDocument();
        break;
      case "file.save_as":
        await saveActiveDocumentAs();
        break;
      case "file.preferences":
        break;
      default:
        break;
    }
  });
}
