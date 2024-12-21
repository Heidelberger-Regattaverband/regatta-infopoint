import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";
import { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";

/**
* @namespace de.regatta_hd.infoportal.controller
*/
export default class Kiosk extends BaseController {

  formatter: Formatter = Formatter;
  private indexFinished: number;
  private indexNext: number;
  private readonly heatFinishedModel: JSONModel = new JSONModel();
  private readonly heatNextModel: JSONModel = new JSONModel();
  private readonly kioskModel: JSONModel = new JSONModel();
  private readonly finishedModel: JSONModel = new JSONModel();
  private readonly nextModel: JSONModel = new JSONModel();
  private intervalId?: number;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    this.indexFinished = 0;
    this.indexNext = 0;

    super.setViewModel(this.heatFinishedModel, "heatFinished");
    super.setViewModel(this.heatNextModel, "heatNext");
    super.setViewModel(this.finishedModel, "regsFinished");
    super.setViewModel(this.nextModel, "regsNext");
    super.setViewModel(this.kioskModel, "kiosk");

    super.getRouter()?.getRoute("kiosk")?.attachMatched((_: Route$MatchedEvent) => {
      this.loadKioskModel();
      this.intervalId = setInterval(this.updateModels.bind(this), 15000);
    }, this);
  }

  onNavBack(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      delete this.intervalId;
    }
    this.navBack("startpage");
  }

  onRefreshButtonPress(event: Button$PressEvent) {
    this.loadKioskModel();
  }

  private async updateModels(): Promise<void> {
    const data: any = this.kioskModel.getData();
    this.heatFinishedModel.setData(data.finished[this.indexFinished]);
    this.heatNextModel.setData(data.next[this.indexNext]);

    const promises: Promise<void>[] = [];
    if (data.finished && data.finished.length > 0) {
      promises.push(this.loadRegsFinishedModel(data.finished[this.indexFinished].id));
    }
    if (data.next && data.next.length > 0) {
      promises.push(this.loadRegsNextModel(data.next[this.indexNext].id));
    }

    if (promises.length > 0) {
      await Promise.all(promises);

      this.indexFinished += 1;
      this.indexNext += 1;
      if (this.indexFinished >= this.kioskModel.getData().finished.length) {
        this.indexFinished = 0;
      }
      if (this.indexNext >= this.kioskModel.getData().next.length) {
        this.indexNext = 0;
      }
    }
  }

  private async loadRegsFinishedModel(heatId: number): Promise<void> {
    await super.updateJSONModel(this.finishedModel, this.getRegistrationsUrl(heatId), super.getView());
  }

  private async loadRegsNextModel(iHeatId: number): Promise<void> {
    await super.updateJSONModel(this.nextModel, this.getRegistrationsUrl(iHeatId), super.getView());
  }

  private async loadKioskModel(): Promise<void> {
    await super.updateJSONModel(this.kioskModel, await this.getKioskUrl(), super.getView());
    this.updateModels();
  }

  private async getKioskUrl(): Promise<string> {
    const regatta: any = await super.getActiveRegatta();
    return `/api/regattas/${regatta.id}/kiosk`;
  }

  private getRegistrationsUrl(heatId: number): string {
    return `/api/heats/${heatId}`;
  }
}