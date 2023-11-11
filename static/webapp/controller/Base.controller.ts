import Controller from "sap/ui/core/mvc/Controller";
import History from "sap/ui/core/routing/History";
import JSONModel from "sap/ui/model/json/JSONModel";
import EventBus from "sap/ui/core/EventBus";
import Model from "sap/ui/model/Model";
import View from "sap/ui/core/mvc/View";
import Component from "sap/ui/core/Component";
import Router from "sap/ui/core/routing/Router";
import Control from "sap/ui/core/Control";
import UIComponent from "sap/ui/core/UIComponent";
import MyComponent from "de/regatta_hd/Component";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class BaseController extends Controller {

  /**
   * Convenience method for accessing the event bus for this component.
   * @returns {sap.ui.core.EventBus} the event bus for this component
   */
  getEventBus(): EventBus | undefined {
    return super.getOwnerComponent()?.getEventBus();
  }

  /**
   * Convenience method for accessing a component model.
   * @param {string} [name] the model name
   * @returns {sap.ui.model.Model} the model instance
   */
  getComponentModel(name: string): Model | undefined {
    return super.getOwnerComponent()?.getModel(name);
  }

  /**
   * Convenience method for setting the view model.
   * @param {sap.ui.model.Model} model the model instance
   * @param {string} name the model name
   * @returns {sap.ui.mvc.View} the view instance
   */
  setComponentModel(model: Model, name: string): Component | undefined {
    return super.getOwnerComponent()?.setModel(model, name);
  }

  /**
   * Convenience method for accessing the router.
   * @returns {sap.ui.core.routing.Router} the router for this component
   */
  getRouter(): Router {
    return (super.getOwnerComponent() as UIComponent).getRouter();
  }

  /**
   * Convenience method for getting the view model by name.
   * @param {string} [name] the model name
   * @returns {sap.ui.model.Model} the model instance
   */
  getViewModel(name: string): Model | undefined {
    return super.getView()?.getModel(name);
  }

  /**
   * Convenience method for setting the view model.
   * @param {sap.ui.model.Model} model the model instance
   * @param {string} name the model name
   * @returns {sap.ui.mvc.View} the view instance
   */
  setViewModel(model: Model, name: string): View | undefined {
    return super.getView()?.setModel(model, name);
  }

  navBack(target: string): void {
    const previousHash: string | undefined = History.getInstance().getPreviousHash();
    if (previousHash) {
      window.history.go(-1);
    } else {
      this.getRouter().navTo(target, {}, undefined, true /* no history*/);
    }
  }

  displayTarget(target: string): void {
    this.getRouter()?.getTargets()?.display(target);
  }

  i18n(key: string, args?: any[]): string | undefined {
    return (super.getOwnerComponent() as MyComponent).getResourceBundle().getText(key, args);
  }

  getRegattaId(): number {
    return (super.getOwnerComponent() as MyComponent).getRegattaId();
  }

  async createJSONModel(url: string, control?: Control): Promise<JSONModel> {
    return await this.updateJSONModel(new JSONModel(), url, control);
  }

  async updateJSONModel(model: JSONModel, url: string, control?: Control): Promise<JSONModel> {
    control?.setBusy(true);
    await model.loadData(url);
    control?.setBusy(false);
    return model;
  }

}
