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
  private heatFinishedModel: JSONModel;
  private heatNextModel: JSONModel;
  private kioskModel: JSONModel;
  private finishedModel: JSONModel;
  private nextModel: JSONModel;
  private intervalId?: number;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    this.indexFinished = 0;
    this.indexNext = 0;

    this.heatFinishedModel = new JSONModel();
    super.setViewModel(this.heatFinishedModel, "heatFinished");
    this.heatNextModel = new JSONModel();
    super.setViewModel(this.heatNextModel, "heatNext");

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
    if (!this.finishedModel) {
      this.finishedModel = await super.createJSONModel(this.getRegistrationsUrl(heatId), super.getView());
      super.setViewModel(this.finishedModel, "regsFinished");
    } else {
      await super.updateJSONModel(this.finishedModel, this.getRegistrationsUrl(heatId), super.getView());
    }
  }

  private async loadRegsNextModel(iHeatId: number): Promise<void> {
    if (!this.nextModel) {
      this.nextModel = await super.createJSONModel(this.getRegistrationsUrl(iHeatId), super.getView());
      super.setViewModel(this.nextModel, "regsNext");
    } else {
      await super.updateJSONModel(this.nextModel, this.getRegistrationsUrl(iHeatId), super.getView());
    }
  }

  private async loadKioskModel(): Promise<void> {
    if (!this.kioskModel) {
      this.kioskModel = await super.createJSONModel(this.getKioskUrl(), super.getView());
      super.setViewModel(this.kioskModel, "kiosk");
    } else {
      await super.updateJSONModel(this.kioskModel, this.getKioskUrl(), super.getView());
    }

    this.updateModels();
  }

  private getKioskUrl(): string {
    return `/api/regattas/${super.getRegattaId()}/kiosk`;
  }

  private getRegistrationsUrl(heatId: number): string {
    return `/api/heats/${heatId}`;
  }

}